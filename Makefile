.PHONY: serve dev

serve:
	BEVY_ASSET_ROOT="." dx serve --hot-patch --features "bevy/hotpatching"

dev: serve

