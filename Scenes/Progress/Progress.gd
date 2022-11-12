extends VBoxContainer

# warning-ignore:unused_signal
signal show_progress
# warning-ignore:unused_signal
signal progress_change

onready var progress_label = get_node("Label")
onready var progress_bar = get_node("ProgressBar")
onready var progress_log = get_node("ProgressLog")

func _ready():
	# warning-ignore:return_value_discarded
	connect("show_progress", self, "show_progress_handler")
	# warning-ignore:return_value_discarded
	connect("progress_change", self, "progress_change_handler")
	
func show_progress_handler(_data):
	self.visible = true
	progress_bar.visible = true
	get_node("../TitleBar").emit_signal("mode_changed", "progress")
	get_node("../Controls").emit_signal("mode_changed", "progress", "progress")
	
func progress_change_handler(change_str):
	var change = parse_json(change_str)
	
	if change.label:
		progress_label.text = change.label
		progress_bar.value = 0
	elif change.progress:
		progress_bar.value = change.progress
		
	if change.complete:
		show_progress_handler(null)
		progress_bar.visible = false
		progress_label.text = "Download Complete"
		
	if change.log_line:
		if !progress_log.visible:
			progress_log.visible = true
		progress_log.text += change.log_line + "\n"
