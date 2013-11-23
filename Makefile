tmp=_git_distcheck
nrays_doc_path=doc
nrays_rs=src/lib.rs
nrays_lib_path=lib
ncollide_lib_path=./lib/ncollide/lib
nalgebra_lib_path=./lib/nalgebra/lib
build_cmd_opt=rustc -Llib -L$(nalgebra_lib_path) -L$(ncollide_lib_path) --out-dir bin --opt-level 3

all:
	mkdir -p $(nrays_lib_path)
	rustc $(nrays_rs) -L$(nalgebra_lib_path) -L$(ncollide_lib_path) --out-dir $(nrays_lib_path) --opt-level 3

deps:
	make -C ./lib/nalgebra
	make -C ./lib/ncollide deps
	make -C ./lib/ncollide

test:
	mkdir -p bin
	$(build_cmd_opt) examples/sphere4d.rs
	$(build_cmd_opt) examples/sphere3d.rs

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
