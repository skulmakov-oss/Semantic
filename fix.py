import re
path = 'examples/experimental/quad_logic_calculator/src/main.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Replace using regex to ignore line endings
text = re.sub(r'#\[derive\(Clone, Copy, PartialEq, Eq\)\]\s*pub enum Action', '#[derive(Clone, Copy, PartialEq, Eq, Debug)]\npub enum Action', text)

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)
