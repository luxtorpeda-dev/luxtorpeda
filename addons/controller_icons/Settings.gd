tool
extends Resource
class_name ControllerSettings

enum Devices {
	LUNA,
	OUYA,
	PS3,
	PS4,
	PS5,
	STADIA,
	STEAM,
	SWITCH,
	JOYCON,
	VITA,
	WII,
	WIIU,
	XBOX360,
	XBOXONE,
	XBOXSERIES,
	STEAM_DECK
}

## Controller type to fallback to if automatic
## controller detection fails
export(Devices) var joypad_fallback = Devices.XBOX360

# Controller deadzone for triggering an icon remap when input
# is analogic (movement sticks or triggers)
export(float, 0.0, 1.0) var joypad_deadzone := 0.5

# Allow mouse movement to trigger an icon remap
export(bool) var allow_mouse_remap := true

# Minimum mouse "instantaneous" movement for
# triggering an icon remap
export(int, 0, 10000) var mouse_min_movement := 200

# Custom asset lookup folder for custom icons
export(String, DIR) var custom_asset_dir := ""

# Custom generic joystick mapper script
export(Script) var custom_mapper : Script
