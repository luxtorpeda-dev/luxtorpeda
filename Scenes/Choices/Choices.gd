extends Node

# warning-ignore:unused_signal
signal choices_found
# warning-ignore:unused_signal
signal choice_picked
# warning-ignore:unused_signal
signal default_choice_clicked

var last_choices = null
var last_index = -1
onready var choice_list = get_node("ScrollContainer/ChoiceList")
onready var default_icon_texture = load("res://Resources/accept-icon.png")

# Called when the node enters the scene tree for the first time.
func _ready():
	# warning-ignore:return_value_discarded
	connect("choices_found", self, "choices_found_handler")
	# warning-ignore:return_value_discarded
	connect("choice_picked", self, "choice_picked_handler")
	# warning-ignore:return_value_discarded
	connect("default_choice_clicked", self, "default_choice_clicked_handler")
	choice_list.grab_focus()
	
func _input(event):
	if self.visible and event is InputEventJoypadMotion and event.axis == JOY_AXIS_1:
		var new_index = last_index
		if event.get_action_strength("ui_down") >= 1:
			new_index += 1
		elif event.get_action_strength("ui_up") >= 1:
			new_index -= 1
		
		if new_index >= 0 and new_index != last_index and new_index < last_choices.size():
			choice_list.select(new_index)
			_on_ChoiceList_item_selected(new_index)
			
		ControllerIcons._set_last_input_type(ControllerIcons.InputType.CONTROLLER)
		choice_list.accept_event()

func choices_found_handler(choices_str):
	last_choices = parse_json(choices_str)
	get_node("../TitleBar").emit_signal("mode_changed", "choice")
	get_node("../Controls").emit_signal("mode_changed", "choice", "choice")
	self.visible = true
	
	for choice in last_choices:
		choice_list.add_item(choice.name)
		
func choice_picked_handler(_choice_str):
	self.visible = false
	
func default_choice_clicked_handler(current_choice, default_choice):
	if default_choice != current_choice:
		default_choice = current_choice
	else:
		default_choice = null
		
	for i in range(last_choices.size()):
		if default_choice and last_choices[i].name == default_choice:
			choice_list.set_item_icon(i, default_icon_texture)
		else:
			choice_list.set_item_icon(i, null)
		
	get_node("../Controls").emit_signal("default_choice_selected", default_choice)

func _on_ChoiceList_item_selected(index):
	var engine_choice = last_choices[index]
	last_index = index
	get_node("../Controls").emit_signal("choice_selected", engine_choice)
	if engine_choice.notices && engine_choice.notices.size():
		var noticeText = ""
		for notice in engine_choice.notices:
			noticeText += "* " + notice + "\n"
		get_node("LabelScrollContainer/Label").text = noticeText
		get_node("LabelScrollContainer").visible = true
		get_node("Separator").visible = true
	else:
		get_node("LabelScrollContainer/Label").text = ""
		get_node("LabelScrollContainer").visible = false
		get_node("Separator").visible = false
