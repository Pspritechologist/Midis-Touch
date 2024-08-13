build() {
	RUSTFLAGS="-Zlocation-detail=none" cargo +nightly build --release --no-default-features --features=$1
}

move() {
	mkdir -p bin/$1
	cp target/release/midis-touch bin/$1/
	# cp target/release/midis-hw-player bin/$1/
	cp target/release/midis-sw-player bin/$1/
}

# Build once for 60kb_font, 100kb_font, 200kb_font, 1mb_font, and 7mb_font.
build 60kb_font
move 60kb
build 100kb_font
move 100kb
build 200kb_font
move 200kb
build 1mb_font
move 1mb
build 7mb_font
move 7mb
