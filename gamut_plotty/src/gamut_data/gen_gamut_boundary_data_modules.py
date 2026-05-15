"""Generate the Gamut boundary datasets for different Illuminants and Observers."""

import pathlib
import sys

try:
    import colour
    import numpy as np
except ImportError:
    print("Missing modules, install with: 'pip install colour-science'")
    sys.exit(1)


illuminants = [
    "D50",
    "D65",
]

observers = {
    "CIE 1931 2 Degree Standard Observer": "2deg_1931",
    "CIE 1964 10 Degree Standard Observer": "10deg_1964",
    "CIE 2015 2 Degree Standard Observer": "2deg_2015",
    "CIE 2015 10 Degree Standard Observer": "10deg_2015",
}

illuminant_lookup_observers = {
    "CIE 1931 2 Degree Standard Observer": "CIE 1931 2 Degree Standard Observer",
    "CIE 1964 10 Degree Standard Observer": "CIE 1964 10 Degree Standard Observer",
    "CIE 2015 2 Degree Standard Observer": "CIE 1931 2 Degree Standard Observer",
    "CIE 2015 10 Degree Standard Observer": "CIE 1964 10 Degree Standard Observer",
}


for illuminant in illuminants:
    for observer in list(observers.keys()):
        print(
            f"Generating Gamut Boundary for Illuminant: {illuminant}, Observer: {observer}..."
        )

        # Get White Point (XYZ) based on Illuminant
        xy = colour.CCS_ILLUMINANTS[illuminant_lookup_observers[observer]][illuminant]
        white_point_XYZ = colour.xyY_to_XYZ(colour.xy_to_xyY(xy))
        print(f"White Point XYZ: {white_point_XYZ}")

        # Get Color Matching Functions (CMFs) based on Observer
        cmfs = colour.MSDS_CMFS[observer]

        # Extract CMF data and convert to XYZ
        x_bar = cmfs.values[:, 0]
        y_bar = cmfs.values[:, 1]
        z_bar = cmfs.values[:, 2]
        xyz_data = np.column_stack((x_bar, y_bar, z_bar))

        # Convert to CIELAB
        lab_data = colour.XYZ_to_Lab(xyz_data, white_point_XYZ)

        # Output Rust code with metadata
        output_lines = []
        output_lines.append(
            f"/// Generated with Illuminant: {illuminant}, Observer: {observer}"
        )

        output_lines.append(
            f"pub const GAMUT_BOUNDARY: [(f64, f64, f64); {len(lab_data)}] = ["
        )

        count = 0
        for l, a, b in lab_data:  # noqa: E741
            if np.isfinite(l) and np.isfinite(a) and np.isfinite(b):
                output_lines.append(f"    ({l:.6f}, {a:.6f}, {b:.6f}),")
                count += 1

        output_lines.append("];")
        print(f"Generated {count} valid points.")

        file_name = f"cie_{illuminant}_{observers[observer]}.rs".casefold()
        (pathlib.Path(__file__).parent / file_name).write_text("\n".join(output_lines))
