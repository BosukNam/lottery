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

// ============================================================
// 🧠 통계 추천 (필터 + 다양성 MMR-style)
// ============================================================

function buildSmartStats(data) {
    const TOTAL = data.length;
    const RECENT_N = 50;
    const last = data[data.length - 1];
    const lastSet = new Set(last.numbers);

    const freq = new Map();
    const recentFreq = new Map();
    const lastSeen = new Map();
    const pairFreq = new Map();
    const pastSets = new Set();

    for (let n = 1; n <= 45; n++) {
        freq.set(n, 0);
        recentFreq.set(n, 0);
    }

    data.forEach((d, i) => {
        const sorted = [...d.numbers].sort((a, b) => a - b);
        pastSets.add(sorted.join(','));
        sorted.forEach(n => {
            freq.set(n, freq.get(n) + 1);
            lastSeen.set(n, i);
        });
        if (i >= TOTAL - RECENT_N) {
            sorted.forEach(n => recentFreq.set(n, recentFreq.get(n) + 1));
        }
        for (let a = 0; a < sorted.length - 1; a++) {
            for (let b = a + 1; b < sorted.length; b++) {
                const k = `${sorted[a]}-${sorted[b]}`;
                pairFreq.set(k, (pairFreq.get(k) || 0) + 1);
            }
        }
    });

    const gap = new Map();
    for (let n = 1; n <= 45; n++) {
        gap.set(n, TOTAL - 1 - (lastSeen.get(n) ?? -1));
    }

    const normalize = (m) => {
        const vals = [...m.values()];
        const mn = Math.min(...vals), mx = Math.max(...vals);
        const out = new Map();
        if (mx === mn) {
            for (const k of m.keys()) out.set(k, 0.5);
        } else {
            for (const [k, v] of m) out.set(k, (v - mn) / (mx - mn));
        }
        return out;
    };

    const fN = normalize(freq);
    const rN = normalize(recentFreq);
    const gN = normalize(gap);

    const W_FREQ = 0.25, W_RECENT = 0.40, W_GAP = 0.35;
    const scoreNum = new Map();
    for (let n = 1; n <= 45; n++) {
        scoreNum.set(n, W_FREQ * fN.get(n) + W_RECENT * rN.get(n) + W_GAP * gN.get(n));
    }

    const maxPairFreq = Math.max(...pairFreq.values(), 1);

    return { TOTAL, last, lastSet, scoreNum, pairFreq, pastSets, maxPairFreq, freq, recentFreq, gap };
}

function passesFilters(combo, stats) {
    const sum = combo.reduce((a, b) => a + b, 0);
    if (sum < 100 || sum > 175) return false;
    const odd = combo.filter(n => n % 2 === 1).length;
    if (![2, 3, 4].includes(odd)) return false;
    const low = combo.filter(n => n <= 22).length;
    if (![2, 3, 4].includes(low)) return false;
    const sorted = [...combo].sort((a, b) => a - b);
    let consec = 0;
    for (let i = 0; i < 5; i++) if (sorted[i + 1] - sorted[i] === 1) consec++;
    if (consec > 1) return false;
    let overlap = 0;
    for (const n of combo) if (stats.lastSet.has(n)) overlap++;
    if (overlap > 1) return false;
    return true;
}

function totalScore(combo, stats) {
    let numS = 0;
    for (const n of combo) numS += stats.scoreNum.get(n);
    numS /= 6;
    const sorted = [...combo].sort((a, b) => a - b);
    let pairS = 0;
    for (let a = 0; a < 5; a++) {
        for (let b = a + 1; b < 6; b++) {
            const k = `${sorted[a]}-${sorted[b]}`;
            pairS += (stats.pairFreq.get(k) || 0);
        }
    }
    pairS = (pairS / 15) / stats.maxPairFreq;
    return numS * 0.6 + pairS * 0.4;
}

function weightedSampleCombo(stats, temperature, rand) {
    const weights = [];
    for (let n = 1; n <= 45; n++) {
        weights.push(Math.pow(stats.scoreNum.get(n) + 0.05, temperature));
    }
    const chosen = new Set();
    let safety = 0;
    while (chosen.size < 6 && safety < 200) {
        safety++;
        const total = weights.reduce((a, b) => a + b, 0);
        let r = rand() * total;
        for (let n = 1; n <= 45; n++) {
            r -= weights[n - 1];
            if (r <= 0) {
                chosen.add(n);
                break;
            }
        }
    }
    return [...chosen].sort((a, b) => a - b);
}

function diverseSelect(cands, stats, target, maxPerNumber, lambdaDiv) {
    const selected = [];
    const usage = new Map();
    const remaining = cands.map((c, i) => ({ c, i, base: totalScore(c, stats) }));
    while (remaining.length > 0 && selected.length < target) {
        let bestIdx = -1, bestScore = -Infinity;
        for (let i = 0; i < remaining.length; i++) {
            const { c, base } = remaining[i];
            if (c.some(n => (usage.get(n) || 0) >= maxPerNumber)) continue;
            let penalty = 0;
            for (const n of c) penalty += (usage.get(n) || 0);
            const adj = base - lambdaDiv * penalty;
            if (adj > bestScore) { bestScore = adj; bestIdx = i; }
        }
        if (bestIdx === -1) {
            // 캡 풀고 재시도
            for (let i = 0; i < remaining.length; i++) {
                const { c, base } = remaining[i];
                let penalty = 0;
                for (const n of c) penalty += (usage.get(n) || 0);
                const adj = base - lambdaDiv * penalty;
                if (adj > bestScore) { bestScore = adj; bestIdx = i; }
            }
            if (bestIdx === -1) break;
        }
        const pick = remaining.splice(bestIdx, 1)[0].c;
        selected.push(pick);
        for (const n of pick) usage.set(n, (usage.get(n) || 0) + 1);
    }
    return { selected, usage };
}

// 시드 가능한 PRNG (mulberry32)
function makeRng(seed) {
    let s = seed >>> 0;
    return function () {
        s = (s + 0x6D2B79F5) >>> 0;
        let t = s;
        t = Math.imul(t ^ (t >>> 15), t | 1);
        t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
}

window.showSmartRecommend = function () {
    const content = document.getElementById('content');
    content.innerHTML = `
        <div class="result-title">🧠 통계 추천</div>
        <div class="form-group">
            <label class="form-label">시드 (다른 추천을 보고 싶으면 변경)</label>
            <input type="number" id="smart-seed" class="form-input" value="42">
        </div>
        <button class="submit-btn" onclick="runSmartRecommend()">10개 조합 생성</button>
        <div id="smart-result"></div>
    `;
};

window.runSmartRecommend = function () {
    const resultDiv = document.getElementById('smart-result');
    resultDiv.innerHTML = '<div class="loading">생성 중...</div>';

    setTimeout(() => {
        try {
            const seed = parseInt(document.getElementById('smart-seed').value) || 42;
            const stats = buildSmartStats(lotteryData);
            const rand = makeRng(seed);

            const seen = new Set();
            const cands = [];
            const temperature = 0.5;
            const ATTEMPTS = 30000;
            const MAX_CANDS = 3000;
            let tried = 0;
            while (tried < ATTEMPTS && cands.length < MAX_CANDS) {
                tried++;
                const combo = weightedSampleCombo(stats, temperature, rand);
                const key = combo.join(',');
                if (seen.has(key)) continue;
                seen.add(key);
                if (!passesFilters(combo, stats)) continue;
                if (stats.pastSets.has(key)) continue;
                cands.push(combo);
            }

            cands.sort((a, b) => totalScore(b, stats) - totalScore(a, stats));
            const pool = cands.slice(0, 800);
            const { selected, usage } = diverseSelect(pool, stats, 10, 4, 0.15);

            let html = `<div class="success-message">시드 ${seed} · 후보 ${cands.length}개 → 추천 ${selected.length}개</div>`;

            selected.forEach((nums, idx) => {
                const sum = nums.reduce((a, b) => a + b, 0);
                const odd = nums.filter(n => n % 2 === 1).length;
                const low = nums.filter(n => n <= 22).length;
                html += `
                    <div class="lottery-set">
                        <div class="lottery-set-header">
                            추천 ${idx + 1} · 합 ${sum} · 홀짝 ${odd}:${6 - odd} · 저고 ${low}:${6 - low}
                        </div>
                        <div class="lottery-numbers">
                            ${nums.map(n => `<div class="lottery-number" style="background: ${getNumberColor(n)}; color: white;">${n}</div>`).join('')}
                        </div>
                    </div>
                `;
            });

            const usageSorted = [...usage.entries()].sort((a, b) => b[1] - a[1]);
            const usageStr = usageSorted.map(([n, c]) => `${n}(${c})`).join(', ');
            html += `<div class="note">
                ※ 필터: 합 100~175, 홀짝/저고 2:4·3:3·4:2, 연속번호 0~1쌍, 직전회차 겹침 0~1<br>
                ※ 다양성: 한 번호 최대 4회 등장 · 사용 고유 번호 ${usage.size}/45<br>
                ※ 번호별 등장: ${usageStr}<br>
                ※ 1~1221회차 당첨조합 모두 제외됨
            </div>`;

            resultDiv.innerHTML = html;
        } catch (error) {
            resultDiv.innerHTML = `<div class="error-message">오류: ${error.message}</div>`;
            console.error(error);
        }
    }, 100);
};

// 초기화
initialize();
