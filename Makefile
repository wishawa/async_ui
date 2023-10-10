# makefile for building + deploying book and demos

.PHONY: book_pages
book_pages:
	@echo "====> making book for gh-pages"
	# before running make, run `git worktree add gh-pages/ gh-pages
	rm -rf gh-pages/book
	mdbook build --dest-dir ../gh-pages/book guide/

.PHONY: demo_todomvc
demo_todomvc:
	@echo "====> building demo todomvc"
	rm -rf gh-pages/demos/todomvc
	wasm-pack build --release --target web --out-dir ../../gh-pages/demos/todomvc/pkg examples/web-todomvc
	rm gh-pages/demos/todomvc/pkg/.gitignore
	cp examples/web-todomvc/index.html gh-pages/demos/todomvc/

.PHONY: demo_simple
demo_simple:
	@echo "====> building demo simple"
	rm -rf gh-pages/demos/simple
	wasm-pack build --release --target web --out-dir ../../gh-pages/demos/simple/pkg examples/web-simple-examples
	rm gh-pages/demos/simple/pkg/.gitignore
	cp examples/web-simple-examples/index.html gh-pages/demos/simple/

.PHONY: demo_x_bow_playground
demo_x_bow_playground:
	@echo "====> building demo x_bow_playground"
	rm -rf gh-pages/demos/x-bow-playground
	wasm-pack build --release --target web --out-dir ../../gh-pages/demos/x-bow-playground/pkg examples/x-bow-playground
	rm gh-pages/demos/x-bow-playground/pkg/.gitignore
	cp examples/x-bow-playground/index.html gh-pages/demos/x-bow-playground/

.PHONY: demo_login_flow
demo_login_flow:
	@echo "====> building demo login_flow"
	rm -rf gh-pages/demos/login-flow
	wasm-pack build --release --target web --out-dir ../../gh-pages/demos/login-flow/pkg examples/login-flow
	rm gh-pages/demos/login-flow/pkg/.gitignore
	cp examples/login-flow/index.html gh-pages/demos/login-flow/

.PHONY: demos
demos: demo_todomvc demo_simple demo_x_bow_playground demo_login_flow