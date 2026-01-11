import init, { LotteryEngine } from './pkg/lottery.js';

let engine = null;
let lotteryData = null;  // 역대 당첨번호 조회용

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
        lotteryData = JSON.parse(jsonData);  // 역대 당첨번호 조회용 저장
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
                            ${numbers.map(num => `<div class="lottery-number" style="background: ${getNumberColor(num)}; color: white;">${num}</div>`).join('')}
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
                                const borderStyle = isRequired ? 'border: 3px solid white;' : '';
                                return `<div class="lottery-number" style="background: ${getNumberColor(num)}; color: white; ${borderStyle}">${num}</div>`;
                            }).join('')}
                        </div>
                    </div>
                `;
            });

            html += '<div class="note">※ 테두리 있는 번호는 지정한 번호입니다<br>※ 기존 1등, 2등 당첨번호 제외</div>';

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
                            <div class="frequency-number" style="background: ${getNumberColor(number)};">${number}</div>
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

// 번호 색상 결정 함수
function getNumberColor(num) {
    if (num <= 10) return '#fbc400';      // 노랑
    if (num <= 20) return '#69c8f2';      // 파랑
    if (num <= 30) return '#ff7272';      // 빨강
    if (num <= 40) return '#aaa';         // 회색
    return '#b0d840';                      // 초록
}

// 역대 당첨번호 조회
window.showWinningNumbers = function() {
    const content = document.getElementById('content');

    if (!lotteryData || lotteryData.length === 0) {
        content.innerHTML = '<div class="error-message">데이터를 불러올 수 없습니다.</div>';
        return;
    }

    // 최신 회차순으로 정렬
    const sortedData = [...lotteryData].sort((a, b) => b.round - a.round);
    const latestRound = sortedData[0].round;

    // 회차 선택 옵션 생성
    const options = sortedData.map(d =>
        `<option value="${d.round}">${d.round}회</option>`
    ).join('');

    content.innerHTML = `
        <div class="result-title">🏆 역대 당첨번호 조회</div>
        <div class="form-group">
            <label class="form-label">회차 선택</label>
            <select id="round-select" class="form-input" onchange="displayWinningNumber()">
                ${options}
            </select>
        </div>
        <div id="winning-result"></div>
    `;

    // 최신 회차 당첨번호 표시
    displayWinningNumber();
};

// 선택된 회차의 당첨번호 표시
window.displayWinningNumber = function() {
    const round = parseInt(document.getElementById('round-select').value);
    const resultDiv = document.getElementById('winning-result');

    const drawing = lotteryData.find(d => d.round === round);

    if (!drawing) {
        resultDiv.innerHTML = '<div class="error-message">해당 회차 정보를 찾을 수 없습니다.</div>';
        return;
    }

    const numbersHtml = drawing.numbers.map(num =>
        `<div class="lottery-number" style="background: ${getNumberColor(num)};">${num}</div>`
    ).join('');

    const bonusHtml = `<div class="lottery-number bonus-number" style="background: ${getNumberColor(drawing.bonus)};">${drawing.bonus}</div>`;

    resultDiv.innerHTML = `
        <div class="winning-info">
            <div class="winning-round">${round}회 당첨번호</div>
            <div class="winning-numbers-container">
                <div class="lottery-numbers">
                    ${numbersHtml}
                </div>
                <div class="bonus-separator">+</div>
                ${bonusHtml}
            </div>
            <div class="bonus-label">보너스</div>
        </div>
    `;
};

// 초기화
initialize();
