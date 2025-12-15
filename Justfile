format-md:
	@echo "Running Prettier (write) on Markdown..."
	prettier --write '**/*.md'

check-md:
	@echo "Checking Markdown formatting with Prettier..."
	prettier --check '**/*.md'
