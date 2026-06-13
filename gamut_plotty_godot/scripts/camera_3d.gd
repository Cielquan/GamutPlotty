extends Camera3D

# Movement Speed
var base_move_speed := 5.0
var sprint_move_speed := 4.0
var mouse_sensitivity := 0.05

# Camera rotation
var pitch := 0.0 # Up/Down
var yaw := 0.0   # Left/Right

func _ready() -> void:
    var start_angles := rotation_degrees
    pitch = start_angles.x
    yaw = start_angles.y

    # Disable the default mouse lock behavior if you want continuous freefly
    # In Godot, we often handle input manually for this style
    Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)

func _input(event: InputEvent) -> void:
    var mouse_event := event as InputEventMouseMotion

    if mouse_event and Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED:
        yaw -= mouse_event.relative.x * mouse_sensitivity
        pitch -= mouse_event.relative.y * mouse_sensitivity
        # Clamp pitch so you don't spin your head backwards unnaturally (-90 to 90 degrees)
        pitch = clamp(pitch, -89.99, +89.99)
        # Apply the rotations to the camera
        rotation_degrees = Vector3(pitch, yaw, 0.0)

func _process(delta: float) -> void:
    # Only handle movement if mouse is captured
    if Input.get_mouse_mode() != Input.MOUSE_MODE_CAPTURED:
        return

    # Handle mouse release
    if Input.is_action_just_pressed("release_mouse"):
            Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE)

    # Get the direction vectors relative to where the camera is facing
    var forward := -global_transform.basis.z
    var right := global_transform.basis.x
    var up := global_transform.basis.y

    var direction := Vector3.ZERO

    # Check movement keys
    if Input.is_action_pressed("move_forward"):
        direction += forward
    if Input.is_action_pressed("move_backward"):
        direction -= forward
    if Input.is_action_pressed("move_left"):
        direction -= right
    if Input.is_action_pressed("move_right"):
        direction += right

    # Normalize to prevent faster diagonal movement
    if direction.length() > 0:
        direction = direction.normalized()

    # Add vertical movement
    if Input.is_action_pressed("move_up"):
        direction += up
    if Input.is_action_pressed("move_down"):
        direction -= up

    var final_move_speed := base_move_speed
    if Input.is_action_pressed("move_fast"):
        final_move_speed *= sprint_move_speed

    # Move the camera
    global_position += direction * final_move_speed * delta
