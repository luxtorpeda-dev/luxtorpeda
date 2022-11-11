extends HBoxContainer

signal choice_selected
var last_choice


# Declare member variables here. Examples:
# var a = 2
# var b = "text"


# Called when the node enters the scene tree for the first time.
func _ready():
	connect("choice_selected", self, "signal_handler")
	
func signal_handler(choice):
	last_choice = choice.name
	get_node("OkButton").disabled = false
	get_node("SecondaryButton").disabled = false


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass

func _on_OkButton_pressed():
	# todo - this needs to be aware of what mode we are in
	get_node("../../SignalEmitter").emit_signal("choice_picked", last_choice)
	get_node("../Choices").emit_signal("choice_picked", last_choice)
	get_node("../Progress").emit_signal("show_progress")
	get_node("../TitleBar").emit_signal("mode_changed", "progress")
	
	get_node("OkButton").visible = false
	get_node("SecondaryButton").visible = false


func _on_CancelButton_pressed():
	get_tree().quit()
