extends VBoxContainer

signal mode_changed

# Called when the node enters the scene tree for the first time.
func _ready():
	connect("mode_changed", self, "mode_changed_handler")
	
func mode_changed_handler(new_mode):
	var new_title = ""
	if new_mode == "progress":
		new_title = "Progress"
	else:
		new_title = new_mode
		
	if new_title:
		get_node("Label").text = new_title
