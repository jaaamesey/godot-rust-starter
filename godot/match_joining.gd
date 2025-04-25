extends Node

@export var host_input: LineEdit
@export var port_input: LineEdit
@export var host_button: Button
@export var join_button: Button
@export var message_label: Label

func _ready() -> void:
	multiplayer.peer_connected.connect(func(id): 
		message_label.text = "Player %s connected" % id
	)
	multiplayer.peer_disconnected.connect(func(id): 
		message_label.text = "Player %s disconnected" % id
	)
	multiplayer.server_disconnected.connect(func(): 
		message_label.text = "Host disconnected"
	)
	
	host_button.pressed.connect(func(): 
		var peer := ENetMultiplayerPeer.new()
		peer.create_server(int(port_input.text))
		multiplayer.multiplayer_peer = peer
	)
	join_button.pressed.connect(func(): 
		var peer := ENetMultiplayerPeer.new()
		peer.create_client(host_input.text.strip_edges(), int(port_input.text))
		multiplayer.multiplayer_peer = peer
	)
