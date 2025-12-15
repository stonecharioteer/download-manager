# Admonitions Guide

This book uses mdbook-admonish to provide beautiful, customizable admonitions with unique caricature sprites. All admonitions work seamlessly across all mdbook themes (Light, Rust, Coal, Navy, Ayu).

## Quick Reference

**Total Types:** 12 unique admonition types
**Sprites:** Web-optimized at 120x120px (~18KB each)
**Syntax:** Clean markdown using ` ```admonish type` blocks

## All Admonition Types

### 1. Note
![](src/sprites/plain_1.png)

Mack's neutral, plain expression for general information and observations.

```markdown
```admonish note
Your note content here
```
```

### 2. Abstract/Summary
![](src/sprites/puzzled_1.png)

Mack's puzzled, thinking expression for summaries and abstracts.

**Aliases:** `summary`, `tldr`

```markdown
```admonish abstract title="Summary"
Your summary content here
```
```

### 3. Info
![](src/sprites/smile_bashful_1.png)

Mack's bashful expression for sharing helpful information.

**Alias:** `todo`

```markdown
```admonish info title="Information"
Your info content here
```
```

### 4. Tip
![](src/sprites/happy_1.png)

Mack's happy expression for helpful suggestions and pro tips.

**Aliases:** `hint`, `important`

```markdown
```admonish tip
Your tip content here
```
```

### 5. Success
![](src/sprites/cry_happy_1.png)

Mack's overjoyed expression when things work perfectly!

**Aliases:** `check`, `done`

```markdown
```admonish success title="Success!"
Your success message here
```
```

### 6. Question
![](src/sprites/amaze_1.png)

Mack's amazed, curious expression for questions and FAQs.

**Aliases:** `help`, `faq`

```markdown
```admonish question title="Question?"
Your question content here
```
```

### 7. Warning
![](src/sprites/yell_angry.png)

Mack's yelling angry expression for critical warnings.

**Aliases:** `caution`, `attention`

```markdown
```admonish warning
Your warning content here
```
```

### 8. Failure
![](src/sprites/cry_sad_1.png)

Mack's sad crying expression when things go wrong.

**Aliases:** `fail`, `missing`

```markdown
```admonish failure title="Failed"
Your failure message here
```
```

### 9. Danger
![](src/sprites/shock_1.png)

Mack's shocked expression for serious warnings about destructive operations.

**Alias:** `error`

```markdown
```admonish danger title="Danger!"
Your danger warning here
```
```

### 10. Bug
![](src/sprites/annoyed_1.png)

Mack's annoyed expression when identifying bugs in the code.

```markdown
```admonish bug title="Bug Found"
Your bug description here
```
```

### 11. Example
![](src/sprites/smile_wicked_1.png)

Mack's wicked smile for demonstrating clever code examples.

```markdown
```admonish example
Your example content here
```
```

### 12. Quote
![](src/sprites/love_1.png)

Mack shares wisdom with a loving expression.

**Alias:** `cite`

```markdown
```admonish quote title="Wisdom"
Your quote content here
```
```

## Syntax Options

All admonitions support:

1. **Default title** (uses the type name):
   ```markdown
   ```admonish note
   Content here
   ```
   ```

2. **Custom title**:
   ```markdown
   ```admonish note title="My Custom Title"
   Content here
   ```
   ```

3. **Collapsible** (user can expand/collapse):
   ```markdown
   ```admonish note collapsible=true
   Content here
   ```
   ```

4. **Combined options**:
   ```markdown
   ```admonish tip title="Pro Tip" collapsible=true
   Content here
   ```
   ```

## Theme Compatibility

All admonitions use mdbook-admonish's built-in theme integration, ensuring they look great in all themes (Light, Rust, Coal, Navy, Ayu). The custom sprites are overlaid using CSS to replace the default icons while maintaining the theme's color scheme.

## See All Types in Action

Visit the [Meet Mack](./meet_mack.md) page to see all 12 admonition types rendered with their unique sprites!
