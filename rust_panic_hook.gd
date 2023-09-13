extends Node

var progress_node

func rust_panic_hook(error_msg: String) -> void:
	print("Rust panic: " + str(error_msg))
	var final_error_message = "Rust Panic:\n" + error_msg
	
	var data = {"error": final_error_message, "label": null, "progress": null, "complete": false, "log_line": null, "prompt_items": null}
	
	if progress_node:
		progress_node.progress_change_handler(JSON.stringify(data))
	else:
		get_tree().quit()
