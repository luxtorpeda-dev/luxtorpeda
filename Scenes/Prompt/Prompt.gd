extends VBoxContainer

signal show_prompt
signal hide_prompt

onready var prompt_label = get_node("PromptLabel")

# Called when the node enters the scene tree for the first time.
func _ready():
	connect("show_prompt", self, "show_prompt_handler")
	connect("hide_prompt", self, "hide_prompt_handler")


func show_prompt_handler(data_str):
	var prompt = parse_json(data_str)
	
	if prompt.label:
		prompt_label.text = prompt.label
		
	get_node("../Controls").emit_signal("mode_changed", prompt.promptType, prompt.promptId)
	get_node("../TitleBar").emit_signal("mode_changed", prompt.title)
	
	self.visible = true

func hide_prompt_handler():
	self.visible = false
