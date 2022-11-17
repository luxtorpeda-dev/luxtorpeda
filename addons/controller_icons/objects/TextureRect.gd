tool
extends TextureRect
class_name ControllerTextureRect

export(String) var path : String = "" setget set_path
export(int, "Both", "Keyboard/Mouse", "Controller") var show_only : int = 0 setget set_show_only
export(int) var max_width : int = 40 setget set_max_width

func _ready():
	ControllerIcons.connect("input_type_changed", self, "_on_input_type_changed")
	set_path(path)
	set_max_width(max_width)

func _on_input_type_changed(input_type):
	if show_only == 0 or \
		(show_only == 1 and input_type == ControllerIcons.InputType.KEYBOARD_MOUSE) or \
		(show_only == 2 and input_type == ControllerIcons.InputType.CONTROLLER):
		visible = true
		set_path(path)
	else:
		visible = false

func set_path(_path: String):
	path = _path
	if is_inside_tree():
		texture = ControllerIcons.parse_path(path)

func set_show_only(_show_only: int):
	show_only = _show_only
	_on_input_type_changed(ControllerIcons._last_input_type)

func set_max_width(_max_width: int):
	max_width = _max_width
	if is_inside_tree():
		if max_width < 0:
			expand = false
		else:
			expand = true
			rect_min_size.x = max_width
			if texture:
				rect_min_size.y = texture.get_height() * max_width / texture.get_width()
			else:
				rect_min_size.y = rect_min_size.x
