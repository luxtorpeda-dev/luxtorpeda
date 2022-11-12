extends Node

# warning-ignore:unused_signal
signal choices_found
# warning-ignore:unused_signal
signal choice_picked

var last_choices = null
onready var choice_list = get_node("ScrollContainer/ChoiceList")

# Called when the node enters the scene tree for the first time.
func _ready():
	# warning-ignore:return_value_discarded
	connect("choices_found", self, "choices_found_handler")
	# warning-ignore:return_value_discarded
	connect("choice_picked", self, "choice_picked_handler")
	choice_list.grab_focus()

func choices_found_handler(choices_str):
	last_choices = parse_json(choices_str)
	get_node("../TitleBar").emit_signal("mode_changed", "choices")
	self.visible = true
	
	for choice in last_choices:
		choice_list.add_item(choice.name)
		
func choice_picked_handler(_choice_str):
	self.visible = false

func _on_ChoiceList_item_selected(index):
	var engine_choice = last_choices[index]
	get_node("../Controls").emit_signal("choice_selected", engine_choice)
	if engine_choice.notices && engine_choice.notices.size():
		var noticeText = ""
		for notice in engine_choice.notices:
			noticeText += notice + "\n"
		get_node("LabelScrollContainer/Label").text = noticeText
		get_node("LabelScrollContainer").visible = true
		get_node("Separator").visible = true
	else:
		get_node("LabelScrollContainer/Label").text = ""
		get_node("LabelScrollContainer").visible = false
		get_node("Separator").visible = false
