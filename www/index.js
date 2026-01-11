import init, { LotteryEngine } from './pkg/lottery.js';

let engine = null;

async function loadLotteryData() {
    try {
        // 캐시 무효화를 위한 타임스탬프 추가 (Safari 대응)
        const cacheBuster = `?t=${Date.now()}`;
        const response = await fetch(`lottery_data.json${cacheBuster}`);
        const data = await response.json();
        return JSON.stringify(data);
    } catch (error) {
        console.error('데이터 로딩 실패:', error);
        throw error;
    }
}

async function initialize() {
    try {
        await init();
        const jsonData = await loadLotteryData();
        engine = new LotteryEngine(jsonData);

        const roundRange = engine.getRoundRange();
        if (roundRange) {
            const [minRound, maxRound, count] = roundRange;
            document.getElementById('round-info').textContent =
                `저장된 회차: ${minRound}회 ~ ${maxRound}회 (총 ${count}개)`;
        }
    } catch (error) {
        document.getElementById('round-info').textContent = '데이터 로딩 실패';
        console.error('초기화 실패:', error);
    }
}

window.showGenerateNumbers = function() {
    const content = document.getElementById('content');
    content.innerHTML = '<div class="loading">번호 생성 중...</div>';

    setTimeout(() => {
        try {
            const sets = engine.generateNumbersSets();

            let html = '<div class="result-title">🎲 추천 로또 번호 5개 세트</div>';

            sets.forEach((numbers, index) => {
                html += `
                    <div class="lottery-set">
                        <div class="lottery-set-header">세트 ${index + 1}</div>
                        <div class="lottery-numbers">
                            ${numbers.map(num => `<div class="lottery-number">${num}</div>`).join('')}
                        </div>
                    </div>
                `;
            });

            html += '<div class="note">※ 기존 1등, 2등 당첨번호 제외</div>';

            content.innerHTML = html;
        } catch (error) {
            content.innerHTML = `<div class="error-message">오류: ${error}</div>`;
        }
    }, 300);
};

window.showGenerateWithRequired = function() {
    const content = document.getElementById('content');
    content.innerHTML = `
        <div class="result-title">🎯 특정 번호 포함 생성</div>
        <form onsubmit="generateWithRequired(event)">
            <div class="form-group">
                <label class="form-label">포함할 번호 (1-6개, 공백으로 구분)</label>
                <input type="text"
                       id="required-numbers"
                       class="form-input"
                       placeholder="예: 7 23 31"
                       required>
            </div>
            <button type="submit" class="submit-btn">번호 생성</button>
        </form>
        <div id="result"></div>
    `;
};

window.generateWithRequired = function(event) {
    event.preventDefault();

    const input = document.getElementById('required-numbers').value;
    const numbers = input.trim().split(/\s+/).map(n => parseInt(n));
    const resultDiv = document.getElementById('result');

    // 유효성 검사
    for (let num of numbers) {
        if (isNaN(num) || num < 1 || num > 45) {
            resultDiv.innerHTML = '<div class="error-message">1-45 사이의 숫자만 입력해주세요.</div>';
            return;
        }
    }

    if (numbers.length === 0 || numbers.length > 6) {
        resultDiv.innerHTML = '<div class="error-message">1-6개의 번호를 입력해주세요.</div>';
        return;
    }

    resultDiv.innerHTML = '<div class="loading">번호 생성 중...</div>';

    setTimeout(() => {
        try {
            const sets = engine.generateNumbersSetsWithRequired(numbers);

            let html = `<div class="success-message">포함된 번호: ${numbers.join(', ')}</div>`;

            sets.forEach((nums, index) => {
                html += `
                    <div class="lottery-set">
                        <div class="lottery-set-header">세트 ${index + 1}</div>
                        <div class="lottery-numbers">
                            ${nums.map(num => {
                                const isRequired = numbers.includes(num);
                                return `<div class="lottery-number" style="${isRequired ? 'background: #ffd700; color: #333;' : ''}">${num}</div>`;
                            }).join('')}
                        </div>
                    </div>
                `;
            });

            html += '<div class="note">※ 금색 번호는 지정한 번호입니다<br>※ 기존 1등, 2등 당첨번호 제외</div>';

            resultDiv.innerHTML = html;
        } catch (error) {
            resultDiv.innerHTML = `<div class="error-message">오류: ${error}</div>`;
        }
    }, 300);
};

window.showFrequency = function() {
    const content = document.getElementById('content');
    content.innerHTML = '<div class="loading">분석 중...</div>';

    setTimeout(() => {
        try {
            const frequency = engine.getNumberFrequency();

            let html = '<div class="result-title">📊 빈도 분석 (낮은 순)</div>';
            html += '<div class="frequency-list">';

            frequency.forEach(([number, count], index) => {
                html += `
                    <div class="frequency-item">
                        <div style="display: flex; align-items: center; gap: 15px;">
                            <div style="font-weight: 600; color: #999; width: 30px;">#${index + 1}</div>
                            <div class="frequency-number">${number}</div>
                        </div>
                        <div class="frequency-count">출현: ${count}회</div>
                    </div>
                `;
            });

            html += '</div>';
            html += '<div class="note">※ 출현 횟수가 적은 번호부터 표시</div>';

            content.innerHTML = html;
        } catch (error) {
            content.innerHTML = `<div class="error-message">오류: ${error}</div>`;
        }
    }, 300);
};

// SHA-256 해시된 관리자 비밀번호 (단방향 암호화)
const ADMIN_PASSWORD_HASH = "02d55d9dd12267248bfb93fa3a1ab0cdd867aa24d8f32cddd185cd4a869408bb";

// SHA-256 해시 함수
async function sha256Hash(str) {
    const encoder = new TextEncoder();
    const data = encoder.encode(str);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    return hashHex;
}

// 비밀번호 확인 후 회차 추가 화면 표시
window.showAddDrawingWithAuth = async function() {
    const content = document.getElementById('content');

    content.innerHTML = `
        <div class="result-title">🔒 관리자 인증</div>
        <div class="form-group">
            <label class="form-label">비밀번호를 입력하세요</label>
            <input type="password"
                   id="admin-password"
                   class="form-input"
                   placeholder="비밀번호"
                   autocomplete="off">
        </div>
        <button onclick="verifyPassword()" class="submit-btn">확인</button>
        <div id="auth-result"></div>
        <div class="note" style="margin-top: 20px;">※ 관리자만 신규 회차를 추가할 수 있습니다</div>
    `;

    // Enter 키로도 확인 가능
    document.getElementById('admin-password').addEventListener('keypress', function(e) {
        if (e.key === 'Enter') {
            verifyPassword();
        }
    });
};

window.verifyPassword = async function() {
    const passwordInput = document.getElementById('admin-password');
    const password = passwordInput.value;
    const resultDiv = document.getElementById('auth-result');

    if (!password) {
        resultDiv.innerHTML = '<div class="error-message">비밀번호를 입력해주세요.</div>';
        return;
    }

    // 입력된 비밀번호를 SHA-256으로 해시하여 비교
    const inputHash = await sha256Hash(password);

    if (inputHash === ADMIN_PASSWORD_HASH) {
        resultDiv.innerHTML = '<div class="success-message">인증 성공! 회차 추가 화면으로 이동합니다...</div>';
        setTimeout(() => {
            showAddDrawing();
        }, 500);
    } else {
        resultDiv.innerHTML = '<div class="error-message">비밀번호가 올바르지 않습니다.</div>';
        passwordInput.value = '';
        passwordInput.focus();
    }
};

window.showAddDrawing = function() {
    const content = document.getElementById('content');
    content.innerHTML = `
        <div class="result-title">➕ 신규 회차 추가</div>
        <form onsubmit="addDrawing(event)">
            <div class="form-group">
                <label class="form-label">회차</label>
                <input type="number"
                       id="round"
                       class="form-input"
                       placeholder="예: 1205"
                       required>
            </div>
            <div class="form-group">
                <label class="form-label">1등 번호 6개 (공백으로 구분)</label>
                <input type="text"
                       id="numbers"
                       class="form-input"
                       placeholder="예: 3 7 12 25 31 44"
                       required>
            </div>
            <div class="form-group">
                <label class="form-label">보너스 번호</label>
                <input type="number"
                       id="bonus"
                       class="form-input"
                       placeholder="예: 15"
                       min="1"
                       max="45"
                       required>
            </div>
            <button type="submit" class="submit-btn">회차 추가</button>
        </form>
        <div id="result"></div>
    `;
};

window.addDrawing = function(event) {
    event.preventDefault();

    const round = parseInt(document.getElementById('round').value);
    const numbersInput = document.getElementById('numbers').value;
    const bonus = parseInt(document.getElementById('bonus').value);
    const resultDiv = document.getElementById('result');

    const numbers = numbersInput.trim().split(/\s+/).map(n => parseInt(n));

    if (numbers.length !== 6) {
        resultDiv.innerHTML = '<div class="error-message">6개의 번호를 입력해주세요.</div>';
        return;
    }

    for (let num of numbers) {
        if (isNaN(num) || num < 1 || num > 45) {
            resultDiv.innerHTML = '<div class="error-message">1-45 사이의 숫자만 입력해주세요.</div>';
            return;
        }
    }

    try {
        engine.addNewDrawing(round, numbers, bonus);

        const roundRange = engine.getRoundRange();
        const [minRound, maxRound, count] = roundRange;

        document.getElementById('round-info').textContent =
            `저장된 회차: ${minRound}회 ~ ${maxRound}회 (총 ${count}개)`;

        resultDiv.innerHTML = `
            <div class="success-message">
                ${round}회차가 추가되었습니다!<br>
                번호: ${numbers.join(', ')} + 보너스: ${bonus}
            </div>
        `;

        // 폼 초기화
        document.getElementById('round').value = '';
        document.getElementById('numbers').value = '';
        document.getElementById('bonus').value = '';

        // LocalStorage에 저장
        localStorage.setItem('lottery_data', engine.exportToJson());

    } catch (error) {
        resultDiv.innerHTML = `<div class="error-message">오류: ${error}</div>`;
    }
};

// 초기화
initialize();
