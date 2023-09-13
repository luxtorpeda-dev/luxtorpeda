@tool
extends Node

signal input_type_changed(input_type)

enum InputType {
	KEYBOARD_MOUSE,
	CONTROLLER
}

var _cached_icons := {}
var _custom_input_actions := {}

var _last_input_type
var _settings

var Mapper = preload("res://addons/controller_icons/Mapper.gd").new()

func _set_last_input_type(__last_input_type):
	_last_input_type = __last_input_type
	emit_signal("input_type_changed", _last_input_type)

func _enter_tree():
	if Engine.is_editor_hint():
		_parse_input_actions()

func _parse_input_actions():
	# Default actions will be the builtin editor actions when
	# the script is at editor ("tool") level. To pickup more
	# actions available, these have to be queried manually
	var keys := [
		"input/ui_accept", "input/ui_cancel", "input/ui_copy",
		"input/ui_cut", "input/ui_down", "input/ui_end",
		"input/ui_filedialog_refresh", "input/ui_filedialog_show_hidden",
		"input/ui_filedialog_up_one_level", "input/ui_focus_next",
		"input/ui_focus_prev", "input/ui_graph_delete",
		"input/ui_graph_duplicate", "input/ui_home",
		"input/ui_left", "input/ui_menu", "input/ui_page_down",
		"input/ui_page_up", "input/ui_paste", "input/ui_redo",
		"input/ui_right", "input/ui_select", "input/ui_swap_input_direction",
		"input/ui_text_add_selection_for_next_occurrence",
		"input/ui_text_backspace", "input/ui_text_backspace_all_to_left",
		"input/ui_text_backspace_all_to_left.macos",
		"input/ui_text_backspace_word", "input/ui_text_backspace_word.macos",
		"input/ui_text_caret_add_above", "input/ui_text_caret_add_above.macos",
		"input/ui_text_caret_add_below", "input/ui_text_caret_add_below.macos",
		"input/ui_text_caret_document_end", "input/ui_text_caret_document_end.macos",
		"input/ui_text_caret_document_start", "input/ui_text_caret_document_start.macos",
		"input/ui_text_caret_down", "input/ui_text_caret_left",
		"input/ui_text_caret_line_end", "input/ui_text_caret_line_end.macos",
		"input/ui_text_caret_line_start", "input/ui_text_caret_line_start.macos",
		"input/ui_text_caret_page_down", "input/ui_text_caret_page_up",
		"input/ui_text_caret_right", "input/ui_text_caret_up",
		"input/ui_text_caret_word_left", "input/ui_text_caret_word_left.macos",
		"input/ui_text_caret_word_right", "input/ui_text_caret_word_right.macos",
		"input/ui_text_clear_carets_and_selection", "input/ui_text_completion_accept",
		"input/ui_text_completion_query", "input/ui_text_completion_replace",
		"input/ui_text_dedent", "input/ui_text_delete",
		"input/ui_text_delete_all_to_right", "input/ui_text_delete_all_to_right.macos",
		"input/ui_text_delete_word", "input/ui_text_delete_word.macos",
		"input/ui_text_indent", "input/ui_text_newline", "input/ui_text_newline_above",
		"input/ui_text_newline_blank", "input/ui_text_scroll_down",
		"input/ui_text_scroll_down.macos", "input/ui_text_scroll_up",
		"input/ui_text_scroll_up.macos", "input/ui_text_select_all",
		"input/ui_text_select_word_under_caret", "input/ui_text_select_word_under_caret.macos",
		"input/ui_text_submit", "input/ui_text_toggle_insert_mode", "input/ui_undo",
		"input/ui_up",
	]
	for key in keys:
		var data : Dictionary = ProjectSettings.get_setting(key)
		if not data.is_empty() and data.has("events") and data["events"] is Array:
			_add_custom_input_action((key as String).trim_prefix("input/"), data)

	# A script running at editor ("tool") level only has
	# the default mappings. The way to get around this is
	# manually parsing the project file and adding the
	# new input actions to lookup.
	var proj_file := ConfigFile.new()
	if proj_file.load("res://project.godot"):
		printerr("Failed to open \"project.godot\"! Custom input actions will not work on editor view!")
		return
	if proj_file.has_section("input"):
		for input_action in proj_file.get_section_keys("input"):
			var data : Dictionary = proj_file.get_value("input", input_action)
			_add_custom_input_action(input_action, data)

func _ready():
	Input.joy_connection_changed.connect(_on_joy_connection_changed)
	_settings = load("res://addons/controller_icons/settings.tres")
	if not _settings:
		_settings = ControllerSettings.new()
	if _settings.custom_mapper:
		Mapper = _settings.custom_mapper.new()
	# Wait a frame to give a chance for the app to initialize
	await get_tree().process_frame
	# Set input type to what's likely being used currently
	if Input.get_connected_joypads().is_empty():
		_set_last_input_type(InputType.KEYBOARD_MOUSE)
	else:
		_set_last_input_type(InputType.CONTROLLER)

func _on_joy_connection_changed(device, connected):
	if device == 0:
		if connected:
			# An await is required, otherwise a deadlock happens
			await get_tree().process_frame
			_set_last_input_type(InputType.CONTROLLER)
		else:
			# An await is required, otherwise a deadlock happens
			await get_tree().process_frame
			_set_last_input_type(InputType.KEYBOARD_MOUSE)

func _input(event: InputEvent):
	var input_type = _last_input_type
	match event.get_class():
		"InputEventKey", "InputEventMouseButton":
			input_type = InputType.KEYBOARD_MOUSE
		"InputEventMouseMotion":
			if _settings.allow_mouse_remap and event.velocity.length() > _settings.mouse_min_movement:
				input_type = InputType.KEYBOARD_MOUSE
		"InputEventJoypadButton":
			input_type = InputType.CONTROLLER
		"InputEventJoypadMotion":
			if abs(event.axis_value) > _settings.joypad_deadzone:
				input_type = InputType.CONTROLLER
	if input_type != _last_input_type:
		_set_last_input_type(input_type)

func _add_custom_input_action(input_action: String, data: Dictionary):
	_custom_input_actions[input_action] = data["events"]

func refresh():
	# All it takes is to signal icons to refresh paths
	emit_signal("input_type_changed", _last_input_type)

func parse_path(path: String, input_type = _last_input_type) -> Texture:
	if typeof(input_type) == TYPE_NIL:
		return null
	var root_paths := _expand_path(path, input_type)
	for root_path in root_paths:
		if not _cached_icons.has(root_path):
			if _load_icon(root_path):
				continue
		return _cached_icons[root_path]
	return null

func parse_path_to_tts(path: String, input_type: int = _last_input_type) -> String:
	if input_type == null:
		return ""
	var tts = _convert_path_to_asset_file(path, input_type)
	return _convert_asset_file_to_tts(tts.get_basename().get_file())

func parse_event(event: InputEvent) -> Texture:
	var path = _convert_event_to_path(event)
	if path.is_empty():
		return null

	var base_paths := [
		_settings.custom_asset_dir + "/",
		"res://addons/controller_icons/assets/"
	]
	for base_path in base_paths:
		if base_path.is_empty():
			continue
		base_path += path + ".png"
		if not _cached_icons.has(base_path):
			if _load_icon(base_path):
				continue
		return _cached_icons[base_path]
	return null

func _expand_path(path: String, input_type: int) -> Array:
	var paths := []
	var base_paths := [
		_settings.custom_asset_dir + "/",
		"res://addons/controller_icons/assets/"
	]
	for base_path in base_paths:
		if base_path.is_empty():
			continue
		base_path += _convert_path_to_asset_file(path, input_type)

		paths.push_back(base_path + ".png")
	return paths

func _is_path_action(path):
	return _custom_input_actions.has(path) or InputMap.has_action(path)

func _convert_path_to_asset_file(path: String, input_type: int) -> String:
	if _is_path_action(path):
		var event = _get_matching_event(path, input_type)
		if event:
			return _convert_event_to_path(event)
		return path
	elif path.substr(0, path.find("/")) == "joypad":
		return Mapper._convert_joypad_path(path, _settings.joypad_fallback)
	else:
		return path

func _convert_asset_file_to_tts(path: String) -> String:
	match path:
		"shift_alt":
			return "shift"
		"esc":
			return "escape"
		"backspace_alt":
			return "backspace"
		"enter_alt":
			return "enter"
		"enter_tall":
			return "keypad enter"
		"arrow_left":
			return "left arrow"
		"arrow_right":
			return "right arrow"
		"del":
			return "delete"
		"arrow_up":
			return "up arrow"
		"arrow_down":
			return "down arrow"
		"shift_alt":
			return "shift"
		"ctrl":
			return "control"
		"kp_add":
			return "keypad plus"
		"mark_left":
			return "left mark"
		"mark_right":
			return "right mark"
		"bracket_left":
			return "left bracket"
		"bracket_right":
			return "right bracket"
		"tilda":
			return "tilde"
		"lb":
			return "left bumper"
		"rb":
			return "right bumper"
		"lt":
			return "left trigger"
		"rt":
			return "right trigger"
		"l_stick_click":
			return "left stick click"
		"r_stick_click":
			return "right stick click"
		"l_stick":
			return "left stick"
		"r_stick":
			return "right stick"
		_:
			return path

func _convert_event_to_path(event: InputEvent):
	if event is InputEventKey:
		# If this is a physical key, convert to localized scancode
		if event.keycode == 0:
			return _convert_key_to_path(DisplayServer.keyboard_get_keycode_from_physical(event.physical_keycode))
		return _convert_key_to_path(event.keycode)
	elif event is InputEventMouseButton:
		return _convert_mouse_button_to_path(event.button_index)
	elif event is InputEventJoypadButton:
		return _convert_joypad_button_to_path(event.button_index)
	elif event is InputEventJoypadMotion:
		return _convert_joypad_motion_to_path(event.axis)

func _convert_key_to_path(scancode: int):
	match scancode:
		KEY_ESCAPE:
			return "key/esc"
		KEY_TAB:
			return "key/tab"
		KEY_BACKSPACE:
			return "key/backspace_alt"
		KEY_ENTER:
			return "key/enter_alt"
		KEY_KP_ENTER:
			return "key/enter_tall"
		KEY_INSERT:
			return "key/insert"
		KEY_DELETE:
			return "key/del"
		KEY_PRINT:
			return "key/print_screen"
		KEY_HOME:
			return "key/home"
		KEY_END:
			return "key/end"
		KEY_LEFT:
			return "key/arrow_left"
		KEY_UP:
			return "key/arrow_up"
		KEY_RIGHT:
			return "key/arrow_right"
		KEY_DOWN:
			return "key/arrow_down"
		KEY_PAGEUP:
			return "key/page_up"
		KEY_PAGEDOWN:
			return "key/page_down"
		KEY_SHIFT:
			return "key/shift_alt"
		KEY_CTRL:
			return "key/ctrl"
		KEY_META:
			match OS.get_name():
				"OSX":
					return "key/command"
				_:
					return "key/meta"
		KEY_ALT:
			return "key/alt"
		KEY_CAPSLOCK:
			return "key/caps_lock"
		KEY_NUMLOCK:
			return "key/num_lock"
		KEY_F1:
			return "key/f1"
		KEY_F2:
			return "key/f2"
		KEY_F3:
			return "key/f3"
		KEY_F4:
			return "key/f4"
		KEY_F5:
			return "key/f5"
		KEY_F6:
			return "key/f6"
		KEY_F7:
			return "key/f7"
		KEY_F8:
			return "key/f8"
		KEY_F9:
			return "key/f9"
		KEY_F10:
			return "key/f10"
		KEY_F11:
			return "key/f11"
		KEY_F12:
			return "key/f12"
		KEY_KP_MULTIPLY, KEY_ASTERISK:
			return "key/asterisk"
		KEY_KP_SUBTRACT, KEY_MINUS:
			return "key/minus"
		KEY_KP_ADD:
			return "key/plus_tall"
		KEY_KP_0:
			return "key/0"
		KEY_KP_1:
			return "key/1"
		KEY_KP_2:
			return "key/2"
		KEY_KP_3:
			return "key/3"
		KEY_KP_4:
			return "key/4"
		KEY_KP_5:
			return "key/5"
		KEY_KP_6:
			return "key/6"
		KEY_KP_7:
			return "key/7"
		KEY_KP_8:
			return "key/8"
		KEY_KP_9:
			return "key/9"
		KEY_UNKNOWN:
			return ""
		KEY_SPACE:
			return "key/space"
		KEY_QUOTEDBL:
			return "key/quote"
		KEY_PLUS:
			return "key/plus"
		KEY_0:
			return "key/0"
		KEY_1:
			return "key/1"
		KEY_2:
			return "key/2"
		KEY_3:
			return "key/3"
		KEY_4:
			return "key/4"
		KEY_5:
			return "key/5"
		KEY_6:
			return "key/6"
		KEY_7:
			return "key/7"
		KEY_8:
			return "key/8"
		KEY_9:
			return "key/9"
		KEY_SEMICOLON:
			return "key/semicolon"
		KEY_LESS:
			return "key/mark_left"
		KEY_GREATER:
			return "key/mark_right"
		KEY_QUESTION:
			return "key/question"
		KEY_A:
			return "key/a"
		KEY_B:
			return "key/b"
		KEY_C:
			return "key/c"
		KEY_D:
			return "key/d"
		KEY_E:
			return "key/e"
		KEY_F:
			return "key/f"
		KEY_G:
			return "key/g"
		KEY_H:
			return "key/h"
		KEY_I:
			return "key/i"
		KEY_J:
			return "key/j"
		KEY_K:
			return "key/k"
		KEY_L:
			return "key/l"
		KEY_M:
			return "key/m"
		KEY_N:
			return "key/n"
		KEY_O:
			return "key/o"
		KEY_P:
			return "key/p"
		KEY_Q:
			return "key/q"
		KEY_R:
			return "key/r"
		KEY_S:
			return "key/s"
		KEY_T:
			return "key/t"
		KEY_U:
			return "key/u"
		KEY_V:
			return "key/v"
		KEY_W:
			return "key/w"
		KEY_X:
			return "key/x"
		KEY_Y:
			return "key/y"
		KEY_Z:
			return "key/z"
		KEY_BRACKETLEFT:
			return "key/bracket_left"
		KEY_BACKSLASH:
			return "key/slash"
		KEY_BRACKETRIGHT:
			return "key/bracket_right"
		KEY_ASCIITILDE:
			return "key/tilda"
		_:
			return ""

func _convert_mouse_button_to_path(button_index: int):
	match button_index:
		MOUSE_BUTTON_LEFT:
			return "mouse/left"
		MOUSE_BUTTON_RIGHT:
			return "mouse/right"
		MOUSE_BUTTON_MIDDLE:
			return "mouse/middle"
		_:
			return "mouse/sample"

func _convert_joypad_button_to_path(button_index: int):
	var path
	match button_index:
		JOY_BUTTON_A:
			path = "joypad/a"
		JOY_BUTTON_B:
			path = "joypad/b"
		JOY_BUTTON_X:
			path = "joypad/x"
		JOY_BUTTON_Y:
			path = "joypad/y"
		JOY_BUTTON_LEFT_SHOULDER:
			path = "joypad/lb"
		JOY_BUTTON_RIGHT_SHOULDER:
			path = "joypad/rb"
		JOY_BUTTON_LEFT_STICK:
			path = "joypad/l_stick_click"
		JOY_BUTTON_RIGHT_STICK:
			path = "joypad/r_stick_click"
		JOY_BUTTON_BACK:
			path = "joypad/select"
		JOY_BUTTON_START:
			path = "joypad/start"
		JOY_BUTTON_DPAD_UP:
			path = "joypad/dpad_up"
		JOY_BUTTON_DPAD_DOWN:
			path = "joypad/dpad_down"
		JOY_BUTTON_DPAD_LEFT:
			path = "joypad/dpad_left"
		JOY_BUTTON_DPAD_RIGHT:
			path = "joypad/dpad_right"
		JOY_BUTTON_GUIDE:
			path = "joypad/home"
		JOY_BUTTON_MISC1:
			path = "joypad/share"
		_:
			return ""
	return Mapper._convert_joypad_path(path, _settings.joypad_fallback)

func _convert_joypad_motion_to_path(axis: int):
	var path : String
	match axis:
		JOY_AXIS_LEFT_X, JOY_AXIS_LEFT_Y:
			path = "joypad/l_stick"
		JOY_AXIS_RIGHT_X, JOY_AXIS_RIGHT_Y:
			path = "joypad/r_stick"
		JOY_AXIS_TRIGGER_LEFT:
			path = "joypad/lt"
		JOY_AXIS_TRIGGER_RIGHT:
			path = "joypad/rt"
		_:
			return ""
	return Mapper._convert_joypad_path(path, _settings.joypad_fallback)

func _get_matching_event(path: String, input_type: int):
	var events : Array
	if _custom_input_actions.has(path):
		events = _custom_input_actions[path]
	else:
		events = InputMap.action_get_events(path)

	for event in events:
		match event.get_class():
			"InputEventKey", "InputEventMouse", "InputEventMouseMotion", "InputEventMouseButton":
				if input_type == InputType.KEYBOARD_MOUSE:
					return event
			"InputEventJoypadButton", "InputEventJoypadMotion":
				if input_type == InputType.CONTROLLER:
					return event
	return null

func _load_icon(path: String) -> int:
	var tex = null
	if path.begins_with("res://"):
		if ResourceLoader.exists(path):
			tex = load(path)
			if not tex:
				return ERR_FILE_CORRUPT
		else:
			return ERR_FILE_NOT_FOUND
	else:
		if not FileAccess.file_exists(path):
			return ERR_FILE_NOT_FOUND
		var img := Image.new()
		var err = img.load(path)
		if err != OK:
			return err
		tex = ImageTexture.new()
		tex.create_from_image(img)
	_cached_icons[path] = tex
	return OK

