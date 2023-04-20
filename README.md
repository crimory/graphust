# graphust

Produce simple graphs from basic text input

## Installation with Homebrew (with custom tap)

```
brew tap crimory/graphust
brew install graphust
```

## Example

### Input
```
A -> B
B -> C
C -> A
```

### Output
```
+---+                   +---+
| A |<------------------| C |
+---+                   +---+
  |                       ^
  |                       |
  |         +---+         |
  --------->| B |---------|
            +---+
```

## Project state
:egg: Alpha: not stable, quite new

## Author's socials
### Youtube
https://www.youtube.com/@MarcinKern
