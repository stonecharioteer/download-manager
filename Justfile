format-md:
	@echo "Running Prettier (write) on Markdown..."
	prettier --write '**/*.md'

check-md:
	@echo "Checking Markdown formatting with Prettier..."
	prettier --check '**/*.md'

serve:
	@echo "Serving the dlm book locally on port 7000..."
	mdbook serve -p 7000 the-dl-book
