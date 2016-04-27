tmp=_git_distcheck

all:
	cargo build --release

render: all
	cd bin; ../target/release/loader3d balls.scene; mv out.png balls.png
	cd bin; ../target/release/loader3d barbara.scene; mv out.png barbara.png
	cd bin; ../target/release/loader3d buddha.scene; mv out.png buddha.png
	cd bin; ../target/release/loader3d conference.scene; mv out.png conference.png
	cd bin; ../target/release/loader3d crytek_sponza.scene; mv out.png crytek_sponza.png
	cd bin; ../target/release/loader3d cube.scene; mv out.png cube.png
	cd bin; ../target/release/loader3d cubic_room.scene; mv out.png cubic_room.png
	cd bin; ../target/release/loader3d dabrovic_sponza.scene; mv out.png dabrovic_sponza.png
	cd bin; ../target/release/loader3d dragon.scene; mv out.png dragon.png
	cd bin; ../target/release/loader3d girl.scene; mv out.png girl.png
	cd bin; ../target/release/loader3d hairball.scene; mv out.png hairball.png
	cd bin; ../target/release/loader3d head.scene; mv out.png head.png
	cd bin; ../target/release/loader3d house.scene; mv out.png hous.png
	cd bin; ../target/release/loader3d map.scene; mv out.png map.png
	cd bin; ../target/release/loader3d mitsuba.scene; mv out.png mitsuba.png
	cd bin; ../target/release/loader3d msum.scene; mv out.png msum.png
	cd bin; ../target/release/loader3d powerplant.scene; mv out.png powerplant.png
	cd bin; ../target/release/loader3d primitives.scene; mv out.png primitives.png
	cd bin; ../target/release/loader3d rungholt.scene; mv out.png rungholt.png
	cd bin; ../target/release/loader3d sibenik.scene; mv out.png sibenik.png
	cd bin; ../target/release/loader3d teapot.scene; mv out.png teapot.png
	cd bin; ../target/release/loader3d francois.scene; mv out.png francois.png

clean:
	cargo clean

distcheck:
	rm -rf $(tmp)
	git clone . $(tmp)
	make -C $(tmp)
	rm -rf $(tmp)
