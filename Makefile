.PHONY: book_pages
book_pages:
	@echo "====> making book for gh-pages"
	# before running make, run `git worktree add gh-pages/ gh-pages
	rm -rf gh-pages/book
	mdbook build --dest-dir ../gh-pages/book guide/

.PHONY: demo_todomvc
demo_todomvc:
	@echo "====> building demo todomvc"
	# before running make, run `git worktree add gh-pages/ gh-pages
	rm -rf gh-pages/demos/todomvc
	wasm-pack build --release --target web --out-dir ../../gh-pages/demos/todomvc/pkg examples/web-todomvc
	cp examples/web-todomvc/index.html gh-pages/demos/todomvc/

.PHONY: demo_simple
demo_simple:
	@echo "====> building demo simple"
	# before running make, run `git worktree add gh-pages/ gh-pages
	rm -rf gh-pages/demos/simple
	wasm-pack build --release --target web --out-dir ../../gh-pages/demos/simple/pkg examples/web-simple-examples
	cp examples/web-simple-examples/index.html gh-pages/demos/simple/

.PHONY: demos
demos: demo_todomvc demo_simple