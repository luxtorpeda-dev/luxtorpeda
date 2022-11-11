extends VBoxContainer

signal show_progress
signal progress_change

onready var progress_label = get_node("Label")
onready var progress_bar = get_node("ProgressBar")

# Declare member variables here. Examples:
# var a = 2
# var b = "text"


func _ready():
	connect("show_progress", self, "show_progress_handler")
	connect("progress_change", self, "progress_change_handler")
	
func show_progress_handler():
	self.visible = true
	
func progress_change_handler(change_str):
	var change = parse_json(change_str)
	print(change)
	
	if change.label:
		progress_label.text = change.label
		progress_bar.value = 0
	elif change.progress:
		progress_bar.value = change.progress


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
