import json
import sys
log_path = 'C:/Users/said3/.gemini/antigravity/brain/6104fcc3-36a3-4a39-a219-082677efd3ba/.system_generated/logs/transcript_full.jsonl'
with open(log_path, 'r', encoding='utf-8') as f:
    for line in f:
        try:
            data = json.loads(line)
            if 'tool_calls' in data:
                for call in data['tool_calls']:
                    if call['name'] == 'write_to_file' or call['name'] == 'multi_replace_file_content':
                        if 'TargetFile' in call['args'] and 'main.rs' in call['args']['TargetFile']:
                            print('FOUND:', call['name'])
                            if call['name'] == 'write_to_file':
                                with open('examples/experimental/quad_logic_calculator/src/main.rs', 'w', encoding='utf-8') as out:
                                    out.write(call['args']['CodeContent'])
                            elif call['name'] == 'multi_replace_file_content':
                                pass
        except Exception as e:
            pass
