extends VBoxContainer

# warning-ignore:unused_signal
signal mode_changed

# Called when the node enters the scene tree for the first time.
func _ready():
	# warning-ignore:return_value_discarded
	connect("mode_changed", Callable(self, "mode_changed_handler"))
	
func mode_changed_handler(new_mode):
	var new_title = ""
	if new_mode == "progress":
		new_title = "Progress"
	elif new_mode == "choice":
		new_title = "Pick an engine below"
	elif new_mode == "error":
		new_title = "Error"
	else:
		new_title = new_mode
		
	if new_title:
		get_node("Label").text = new_title
