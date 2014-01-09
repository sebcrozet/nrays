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
