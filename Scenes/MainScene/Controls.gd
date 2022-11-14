extends HBoxContainer

# warning-ignore:unused_signal
signal choice_selected
# warning-ignore:unused_signal
signal mode_changed
# warning-ignore:unused_signal
signal default_choice_selected
# warning-ignore:unused_signal
signal simulate_button

var last_choice
var last_mode = "choice"
var last_mode_id
var last_default_choice = null

onready var ok_button = get_node("OkButton")
onready var secondary_button = get_node("SecondaryButton")
onready var cancel_button = get_node("CancelButton")

# Called when the node enters the scene tree for the first time.
func _ready():
	# warning-ignore:return_value_discarded
	connect("choice_selected", self, "signal_handler")
	# warning-ignore:return_value_discarded
	connect("mode_changed", self, "mode_changed_handler")
	# warning-ignore:return_value_discarded
	connect("default_choice_selected", self, "default_choice_selected_handler")
	# warning-ignore:return_value_discarded
	connect("simulate_button", self, "simulate_button_handler")
	
func signal_handler(choice_data):
	last_choice = choice_data.name
	ok_button.disabled = false
	secondary_button.disabled = false
	
func mode_changed_handler(new_mode, new_mode_id):
	last_mode = new_mode
	last_mode_id = new_mode_id
	cancel_button.text = "Cancel"
	
	if new_mode == "choice":
		ok_button.visible = true
		ok_button.disabled = true
		secondary_button.visible = true
		secondary_button.disabled = true
	elif new_mode == "question":
		ok_button.visible = true
		ok_button.disabled = false
		secondary_button.visible = false
	elif new_mode == "progress":
		ok_button.visible = false
		secondary_button.visible = false
	elif new_mode == "error":
		ok_button.visible = true
		ok_button.disabled = false
		secondary_button.visible = false
		get_node("CancelButton").visible = false
	elif new_mode == "default_choice":
		ok_button.visible = false
		secondary_button.visible = false
		cancel_button.visible = true
		cancel_button.text = "Clear Default"
		
	if cancel_button.visible and !ok_button.visible:
		cancel_button.size_flags_horizontal |= SIZE_EXPAND
		cancel_button.size_flags_horizontal |= SIZE_SHRINK_END
	else:
		cancel_button.size_flags_horizontal = SIZE_SHRINK_END
		
func default_choice_selected_handler(new_default_choice):
	last_default_choice = new_default_choice
	
func simulate_button_handler(button):
	if button == "ok":
		_on_OkButton_pressed()

func _on_OkButton_pressed():
	if last_mode == "choice":
		var choice_picked_obj = {"engine_choice": last_choice, "default_engine_choice": last_default_choice}
		get_node("../../SignalEmitter").emit_signal("choice_picked", to_json(choice_picked_obj))
		get_node("../Choices").emit_signal("choice_picked", last_choice)
	elif last_mode == "question":
		get_node("../Prompt").emit_signal("hide_prompt")
		get_node("../../SignalEmitter").emit_signal("question_confirmed", last_mode_id)
	elif last_mode == "error":
		_on_CancelButton_pressed()
	elif last_mode == "default_choice":
		get_node("../Prompt").emit_signal("hide_prompt")
		var choice_picked_obj = {"engine_choice": last_choice, "default_engine_choice": last_default_choice}
		get_node("../../SignalEmitter").emit_signal("choice_picked", to_json(choice_picked_obj))

func _on_CancelButton_pressed():
	if last_mode == "default_choice":
		get_node("../../SignalEmitter").emit_signal("clear_default_choice", "")
		get_node("../Prompt").emit_signal("hide_prompt")
	else:
		get_tree().quit()

func _on_SecondaryButton_pressed():
	if last_mode == "choice":
		get_node("../Choices").emit_signal("default_choice_clicked", last_choice, last_default_choice)
