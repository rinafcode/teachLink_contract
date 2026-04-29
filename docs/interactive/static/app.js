document.addEventListener('DOMContentLoaded', function() {
    const deployBtn = document.getElementById('deploy-btn');
    const runBtn = document.getElementById('run-btn');
    const invokeBtn = document.getElementById('invoke-btn');
    const functionSelect = document.getElementById('function-select');
    const argsContainer = document.getElementById('args-container');
    const output = document.getElementById('output');
    const invokeResult = document.getElementById('invoke-result');
    const contractStatus = document.getElementById('contract-status');

    let contractId = null;
    let apiSpec = null;

    // Load API spec
    fetch('/api')
        .then(res => res.json())
        .then(data => {
            apiSpec = data;
            populateFunctionSelect();
        });

    deployBtn.addEventListener('click', async () => {
        try {
            const res = await fetch('/deploy', { method: 'POST' });
            const data = await res.json();
            contractId = data.contract_id;
            contractStatus.textContent = `Contract deployed: ${contractId}`;
        } catch (e) {
            contractStatus.textContent = 'Deployment failed';
        }
    });

    runBtn.addEventListener('click', async () => {
        const code = document.getElementById('code-input').value;
        // For now, just display the code
        output.textContent = `Running code: ${code}`;
    });

    invokeBtn.addEventListener('click', async () => {
        if (!contractId) {
            invokeResult.textContent = 'Deploy contract first';
            return;
        }
        const func = functionSelect.value;
        const args = [];
        // Collect args from inputs
        const inputs = argsContainer.querySelectorAll('input');
        inputs.forEach(input => args.push(input.value));

        try {
            const res = await fetch('/invoke', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ function: func, args })
            });
            const data = await res.json();
            invokeResult.textContent = data.result;
        } catch (e) {
            invokeResult.textContent = 'Invocation failed';
        }
    });

    functionSelect.addEventListener('change', () => {
        const func = apiSpec.functions.find(f => f.name === functionSelect.value);
        if (func) {
            argsContainer.innerHTML = '';
            func.args.forEach(arg => {
                const input = document.createElement('input');
                input.placeholder = arg;
                argsContainer.appendChild(input);
            });
        }
    });

    function populateFunctionSelect() {
        functionSelect.innerHTML = '';
        apiSpec.functions.forEach(func => {
            const option = document.createElement('option');
            option.value = func.name;
            option.textContent = func.name;
            functionSelect.appendChild(option);
        });
    }

    // Initialize Mermaid
    mermaid.initialize({ startOnLoad: true });

    // Example diagram
    const diagram = `
    graph TD
        A[User] --> B[Bridge Contract]
        B --> C[Escrow]
        B --> D[Rewards]
        C --> E[Multi-sig]
    `;
    document.getElementById('mermaid-diagram').innerHTML = diagram;
    mermaid.init(undefined, document.getElementById('mermaid-diagram'));
});