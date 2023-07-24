.PHONY: book_pages
book_pages:
	@echo "====> making book for gh-pages"
	# before running make, run `git worktree add gh-pages/ gh-pages
	mdbook build --dest-dir ../gh-pages/book guide/
