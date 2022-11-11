extends VBoxContainer

signal mode_changed

# Declare member variables here. Examples:
# var a = 2
# var b = "text"


# Called when the node enters the scene tree for the first time.
func _ready():
	connect("mode_changed", self, "mode_changed_handler")
	
func mode_changed_handler(new_mode):
	var new_title = ""
	if new_mode == "progress":
		new_title = "Progress"
		
	if new_title:
		get_node("Label").text = new_title


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
