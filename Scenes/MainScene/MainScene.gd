extends Control

var app
var choices = []

var downloads = []
var downloading = false
var download_idx = -1

# Called when the node enters the scene tree for the first time.
func _ready():
	OS.set_window_size(Vector2(1024,600))
	$ItemList.grab_focus()
	$HTTPRequest.request("https://luxtorpeda-dev.github.io/packagesruntime.json")


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass

func _physics_process(delta):
	if downloading:
		print($HTTPRequestDownload.get_body_size())
		print($HTTPRequestDownload.get_downloaded_bytes())
		
		if $HTTPRequestDownload.get_body_size() != -1:
			var percent = (float($HTTPRequestDownload.get_downloaded_bytes()) / float($HTTPRequestDownload.get_body_size())) * 100
			$ProgressBar.value = percent


func _on_HTTPRequest_request_completed(result, response_code, headers, body):
	var apps = parse_json(body.get_string_from_utf8())
	app = apps["2280"]
	var app_choices = app.choices
	downloads = app.download
	
	for choice in app_choices:
		choices.append(choice)
		get_node("ItemList").add_item(choice.name)


func _on_Button_pressed():
	$ItemList.grab_focus()
	start_download()

func start_download():
	download_idx += 1
	if download_idx >= downloads.size():
		$Controls.visible = true
		$ProgressBar.visible = false
		return
		
	$Controls.visible = false
	$ProgressBar.value = 0
	$ProgressBar.visible = true
	
	downloading = true
	var download = downloads[download_idx]
	$HTTPRequestDownload.request(download.url + download.file)

func _on_ItemList_item_selected(index):
	var engine_choice = choices[index]
	print(engine_choice)


func _on_HTTPRequestDownload_request_completed(result, response_code, headers, body):
	downloading = false
	$ProgressBar.value = 100
	
	start_download()
