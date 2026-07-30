import re
path = 'examples/experimental/quad_logic_calculator/src/main.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Using regex to inject the next state proof
text = re.sub(
    r'self\.state = decode_semantic_state\(&next_state_val\)\?;\s*Ok\(\)\s*\}',
    'self.state = decode_semantic_state(&next_state_val)?;\n        println!("[PROOF] Structured next Semantic state: {:?}", self.state);\n        Ok()\n    }',
    text
)

text = re.sub(
    r'frame\s*\}\s*pub fn run',
    'println!("[PROOF] Visible UI update rendered! (Display: \'{}\', Eval: \'{}\')", display_text, eval_str);\n        frame\n    }\n\n    pub fn run',
    text
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)
