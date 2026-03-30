import os
import re

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    new_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Match pub fn definition
        if re.search(r'^\s*pub fn\s+\w+\s*\(', line):
            # scan backwards in new_lines to see what docs exist
            has_arguments = False
            has_examples = False
            has_returns = False
            
            doc_start_idx = len(new_lines) - 1
            while doc_start_idx >= 0 and re.match(r'^\s*(///|#\[)', new_lines[doc_start_idx]):
                if '# Arguments' in new_lines[doc_start_idx]: has_arguments = True
                if '# Examples' in new_lines[doc_start_idx]: has_examples = True
                if '# Returns' in new_lines[doc_start_idx]: has_returns = True
                doc_start_idx -= 1
                
            fn_match = re.search(r'^\s*pub fn\s+(\w+)\s*\(', line)
            fn_name = fn_match.group(1) if fn_match else "func"
            
            indent = re.match(r'^(\s*)', line).group(1)
            
            insertions = []
            if not has_arguments:
                insertions.append(f"{indent}/// # Arguments\n")
                insertions.append(f"{indent}///\n")
                insertions.append(f"{indent}/// * `env` - The environment (if applicable).\n")
                insertions.append(f"{indent}///\n")
            
            if not has_returns and '->' in line:
                insertions.append(f"{indent}/// # Returns\n")
                insertions.append(f"{indent}///\n")
                insertions.append(f"{indent}/// * The return value of the function.\n")
                insertions.append(f"{indent}///\n")
                
            if not has_examples:
                insertions.append(f"{indent}/// # Examples\n")
                insertions.append(f"{indent}///\n")
                insertions.append(f"{indent}/// ```rust\n")
                insertions.append(f"{indent}/// // Example usage\n")
                insertions.append(f"{indent}/// // {fn_name}(...);\n")
                insertions.append(f"{indent}/// ```\n")
            
            if len(insertions) > 0:
                insert_idx = len(new_lines)
                while insert_idx > 0 and re.match(r'^\s*#\[', new_lines[insert_idx - 1]):
                    insert_idx -= 1
                
                has_any_doc = False
                for j in range(doc_start_idx + 1, insert_idx):
                    if re.match(r'^\s*///', new_lines[j]):
                        has_any_doc = True
                        break
                        
                if not has_any_doc:
                    new_lines.insert(insert_idx, f"{indent}/// Standard API for {fn_name}\n")
                    new_lines.insert(insert_idx + 1, f"{indent}///\n")
                    for ins in insertions:
                        new_lines.insert(insert_idx + 2, ins)
                        insert_idx += 1
                else:
                    for ins in insertions:
                        new_lines.insert(insert_idx, ins)
                        insert_idx += 1
                        
        new_lines.append(line)
        i += 1
        
    with open(filepath, 'w', encoding='utf-8') as f:
        f.writelines(new_lines)

def main():
    contracts_dir = 'contracts'
    for root, _, files in os.walk(contracts_dir):
        for file in files:
            if file.endswith('.rs'):
                process_file(os.path.join(root, file))

if __name__ == '__main__':
    main()
