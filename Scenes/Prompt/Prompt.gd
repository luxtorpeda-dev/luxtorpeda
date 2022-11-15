extends VBoxContainer

# warning-ignore:unused_signal
signal show_prompt
# warning-ignore:unused_signal
signal hide_prompt
# warning-ignore:unused_signal
signal show_prompts
# warning-ignore:unused_signal
signal clipboard_paste

onready var prompt_label = get_node("PromptLabel")
onready var prompt_rich_text = get_node("PromptRichText")
onready var text_edit = get_node("TextEdit")
onready var timer = get_node("Timer")

var DEFAULT_TIMER_START = 4
var timer_left = DEFAULT_TIMER_START
var last_prompt = null
var last_prompts = null
var last_prompt_index = 0
var last_prompts_id

# Called when the node enters the scene tree for the first time.
func _ready():
	# warning-ignore:return_value_discarded
	connect("show_prompt", self, "show_prompt_handler")
	# warning-ignore:return_value_discarded
	connect("hide_prompt", self, "hide_prompt_handler")
	# warning-ignore:return_value_discarded
	connect("show_prompts", self, "show_prompts_handler")
	# warning-ignore:return_value_discarded
	connect("clipboard_paste", self, "clipboard_paste_handler")
	

func show_prompt_handler(data_str):
	var prompt = parse_json(data_str)
	process_show_prompt(prompt)
	
func show_prompts_handler(prompts):
	last_prompts = prompts.prompt_items
	last_prompts_id = prompts.prompt_id
	process_show_prompt(last_prompts[last_prompt_index])
	last_prompt_index += 1
	
func process_show_prompt(prompt):
	last_prompt = prompt
	
	if prompt.label:
		prompt_label.text = prompt.label
		
	if prompt.rich_text:
		prompt_rich_text.text = prompt.rich_text
		prompt_rich_text.visible = true
		prompt_label.size_flags_vertical -= SIZE_EXPAND
	else:
		prompt_rich_text.text = ''
		prompt_rich_text.visible = false
		prompt_label.size_flags_vertical |= SIZE_EXPAND
		
	get_node("../Controls").emit_signal("mode_changed", prompt.prompt_type, prompt.prompt_id)
	get_node("../TitleBar").emit_signal("mode_changed", prompt.title)
	get_node("../Progress").emit_signal("hide_progress")
	
	if prompt.prompt_type == "default_choice":
		timer_left = DEFAULT_TIMER_START
		change_label_for_timer()
		timer.start()
	
	if prompt.prompt_type == "input":
		text_edit.visible = true
	else:
		text_edit.visible = false
		
	
	self.visible = true

func hide_prompt_handler():
	if last_prompts:
		if last_prompt_index < last_prompts.size():
			process_show_prompt(last_prompts[last_prompt_index])
			last_prompt_index += 1
			return
		else:
			last_prompts = null
			get_node("../../SignalEmitter").emit_signal("question_confirmed", last_prompts_id)
	
	self.visible = false
	timer.stop()
	
func clipboard_paste_handler():
	var clipboard_value = OS.clipboard
	if clipboard_value:
		text_edit.text = clipboard_value

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
