extends VBoxContainer

# warning-ignore:unused_signal
signal show_prompt
# warning-ignore:unused_signal
signal hide_prompt

onready var prompt_label = get_node("PromptLabel")
onready var timer = get_node("Timer")

var DEFAULT_TIMER_START = 4
var timer_left = DEFAULT_TIMER_START
var last_prompt = null

# Called when the node enters the scene tree for the first time.
func _ready():
	# warning-ignore:return_value_discarded
	connect("show_prompt", self, "show_prompt_handler")
	# warning-ignore:return_value_discarded
	connect("hide_prompt", self, "hide_prompt_handler")


func show_prompt_handler(data_str):
	var prompt = parse_json(data_str)
	last_prompt = prompt
	
	if prompt.label:
		prompt_label.text = prompt.label
		
	get_node("../Controls").emit_signal("mode_changed", prompt.prompt_type, prompt.prompt_id)
	get_node("../TitleBar").emit_signal("mode_changed", prompt.title)
	get_node("../Progress").emit_signal("hide_progress")
	
	if prompt.prompt_type == "default_choice":
		timer_left = DEFAULT_TIMER_START
		change_label_for_timer()
		timer.start()
	
	self.visible = true

func hide_prompt_handler():
	self.visible = false
	timer.stop()

func change_label_for_timer():
	if last_prompt.prompt_type == "default_choice":
		var new_label = "Launching\n" + last_prompt.label + "\nin\n" + str(timer_left) + " seconds"
		prompt_label.text = new_label

func _on_Timer_timeout():
	timer_left -= 1
	
	if timer_left <= 0:
		timer.stop()
		if last_prompt.prompt_type == "default_choice":
			get_node("../Controls").emit_signal("choice_selected", {"name": last_prompt.label})
			get_node("../Controls").emit_signal("simulate_button", "ok")
	
	change_label_for_timer()
