extends HBoxContainer

# warning-ignore:unused_signal
signal choice_selected
# warning-ignore:unused_signal
signal mode_changed

var last_choice
var last_mode = "choice"
var last_mode_id

# Called when the node enters the scene tree for the first time.
func _ready():
	# warning-ignore:return_value_discarded
	connect("choice_selected", self, "signal_handler")
	# warning-ignore:return_value_discarded
	connect("mode_changed", self, "mode_changed_handler")
	
func signal_handler(choice):
	last_choice = choice.name
	get_node("OkButton").disabled = false
	get_node("SecondaryButton").disabled = false
	
func mode_changed_handler(new_mode, new_mode_id):
	last_mode = new_mode
	last_mode_id = new_mode_id
	if new_mode == "question":
		get_node("OkButton").visible = true
		get_node("OkButton").disabled = false
		get_node("SecondaryButton").visible = false
	elif new_mode == "progress":
		get_node("OkButton").visible = false
		get_node("SecondaryButton").visible = false

func _on_OkButton_pressed():
	if last_mode == "choice":
		get_node("../../SignalEmitter").emit_signal("choice_picked", last_choice)
		get_node("../Choices").emit_signal("choice_picked", last_choice)
	elif last_mode == "question":
		get_node("../Prompt").emit_signal("hide_prompt")
		get_node("../../SignalEmitter").emit_signal("question_confirmed", last_mode_id)

func _on_CancelButton_pressed():
	get_tree().quit()
