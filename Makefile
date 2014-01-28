tmp=_git_distcheck
stb_image_lib_path=lib/rust-stb-image/
nrays_doc_path=doc
nrays_rs=src/lib.rs
nrays_lib_path=lib
ncollide_lib_path=./lib/ncollide/lib
nalgebra_lib_path=./lib/nalgebra/lib
png_lib_path=./lib/rust-png
build_cmd_opt=rustc -Llib -L$(stb_image_lib_path) -L$(png_lib_path) -L$(nalgebra_lib_path) -L$(ncollide_lib_path) --out-dir bin --opt-level 3

all:
	mkdir -p $(nrays_lib_path)
	rustc src/lib3d.rs -L$(stb_image_lib_path) -L$(png_lib_path) -L$(nalgebra_lib_path) -L$(ncollide_lib_path) --cfg dim3 --out-dir $(nrays_lib_path) --opt-level 3
	rustc src/lib4d.rs -L$(stb_image_lib_path) -L$(png_lib_path) -L$(nalgebra_lib_path) -L$(ncollide_lib_path) --cfg dim4 --out-dir $(nrays_lib_path) --opt-level 3

deps:
	cd lib/rust-png; ./configure
	make clean -C lib/rust-png
	make -C lib/rust-png
	cd lib/rust-stb-image; ./configure
	make clean -C lib/rust-stb-image
	make -C lib/rust-stb-image
	make -C ./lib/nalgebra
	make -C ./lib/ncollide deps
	make -C ./lib/ncollide

render:
	cd bin; ./loader_3d balls.scene; mv out.png balls.png
	cd bin; ./loader_3d barbara.scene; mv out.png barbara.png
	cd bin; ./loader_3d buddha.scene; mv out.png buddha.png
	cd bin; ./loader_3d conference.scene; mv out.png conference.png
	cd bin; ./loader_3d crytek_sponza.scene; mv out.png crytek_sponza.png
	cd bin; ./loader_3d cube.scene; mv out.png cube.png
	cd bin; ./loader_3d cubic_room.scene; mv out.png cubic_room.png
	cd bin; ./loader_3d dabrovic_sponza.scene; mv out.png dabrovic_sponza.png
	cd bin; ./loader_3d dragon.scene; mv out.png dragon.png
	cd bin; ./loader_3d francois.scene; mv out.png francois.png
	cd bin; ./loader_3d girl.scene; mv out.png girl.png
	cd bin; ./loader_3d hairball.scene; mv out.png hairball.png
	cd bin; ./loader_3d head.scene; mv out.png head.png
	cd bin; ./loader_3d house.scene; mv out.png hous.png
	cd bin; ./loader_3d map.scene; mv out.png map.png
	cd bin; ./loader_3d mitsuba.scene; mv out.png mitsuba.png
	cd bin; ./loader_3d msum.scene; mv out.png msum.png
	cd bin; ./loader_3d powerplant.scene; mv out.png powerplant.png
	cd bin; ./loader_3d primitives.scene; mv out.png primitives.png
	cd bin; ./loader_3d rungholt.scene; mv out.png rungholt.png
	cd bin; ./loader_3d sibenik.scene; mv out.png sibenik.png
	cd bin; ./loader_3d teapot.scene; mv out.png teapot.png

test:
	mkdir -p bin
	$(build_cmd_opt) examples/loader3d.rs
	$(build_cmd_opt) examples/sphere4d.rs

bench:
	mkdir -p $(nrays_lib_path)
	rustc -L$(nalgebra_lib_path) --test $(nrays_rs) --opt-level 3 -o bench~ && ./bench~ --bench
	rm bench~

distcheck:
	rm -rf $(tmp)
	git clone --recursive . $(tmp)
	make -C $(tmp) deps
	make -C $(tmp)
	make -C $(tmp) test
	rm -rf $(tmp)

doc:
	mkdir -p $(nrays_doc_path)
	rustdoc html src/lib.rs -L$(nalgebra_lib_path) -L$(ncollide_lib_path)

.PHONY:doc
.PHONY:test
.PHONY:bench
