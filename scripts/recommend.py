"""
로또 추천 조합 생성기
- 1단계: 통계 필터 통과 (합계/홀짝/저고/연속/직전회차 겹침)
- 3단계: 점수화 (전체빈도 + 최근50 핫 + 미출현 갭 + 페어 동시출현)
- 2단계: 과거 1221회차와 6개 모두 일치하는 조합 제외
"""
import json
import random
from collections import Counter
from itertools import combinations
from pathlib import Path

DATA_PATH = Path(__file__).resolve().parent.parent / "lottery_data.json"

with open(DATA_PATH) as f:
    DATA = json.load(f)

TOTAL = len(DATA)
LAST = DATA[-1]
LAST_SET = set(LAST["numbers"])

# ---- 통계 사전 계산 ----
freq = Counter()
recent_freq = Counter()
last_seen = {}
pair_freq = Counter()
past_sets = set()

RECENT_N = 50
for i, d in enumerate(DATA):
    nums = d["numbers"]
    past_sets.add(tuple(sorted(nums)))
    for n in nums:
        freq[n] += 1
        last_seen[n] = i
    if i >= TOTAL - RECENT_N:
        for n in nums:
            recent_freq[n] += 1
    for a, b in combinations(sorted(nums), 2):
        pair_freq[(a, b)] += 1

gap = {n: TOTAL - 1 - last_seen.get(n, -1) for n in range(1, 46)}

# ---- 번호별 점수 (정규화 후 가중합) ----
def normalize(d):
    vals = list(d.values()) if isinstance(d, dict) else d
    mn, mx = min(vals), max(vals)
    if mx == mn:
        return {k: 0.5 for k in d}
    return {k: (v - mn) / (mx - mn) for k, v in d.items()}

freq_n = normalize({n: freq.get(n, 0) for n in range(1, 46)})
recent_n = normalize({n: recent_freq.get(n, 0) for n in range(1, 46)})
gap_n = normalize({n: gap.get(n, 0) for n in range(1, 46)})  # 갭이 클수록 콜드 회귀 기대

# 가중치: 전체빈도 0.25, 최근핫 0.40, 콜드회귀 0.35
W_FREQ, W_RECENT, W_GAP = 0.25, 0.40, 0.35
score_num = {
    n: W_FREQ * freq_n[n] + W_RECENT * recent_n[n] + W_GAP * gap_n[n]
    for n in range(1, 46)
}

def pair_score(combo):
    s = 0.0
    for a, b in combinations(sorted(combo), 2):
        s += pair_freq.get((a, b), 0)
    # 정규화: 15페어 평균
    return s / 15

# ---- 필터 ----
def passes_filters(combo):
    s = sum(combo)
    if not (100 <= s <= 175):
        return False
    odd = sum(1 for n in combo if n % 2 == 1)
    if odd not in (2, 3, 4):  # 홀짝 2:4, 3:3, 4:2 만 허용
        return False
    low = sum(1 for n in combo if n <= 22)
    if low not in (2, 3, 4):
        return False
    s_combo = sorted(combo)
    consec = sum(1 for i in range(5) if s_combo[i + 1] - s_combo[i] == 1)
    if consec > 1:
        return False
    overlap = len(set(combo) & LAST_SET)
    if overlap > 1:
        return False
    return True

# ---- 가중 샘플링으로 후보 생성 ----
def sample_combo(rng):
    nums = list(range(1, 46))
    weights = [score_num[n] + 0.05 for n in nums]  # 0 가중치 방지
    return tuple(sorted(rng.sample(nums, 6)))  # 균등 샘플링 fallback

def weighted_sample_combo(rng):
    pool = list(range(1, 46))
    weights = [score_num[n] + 0.05 for n in pool]
    chosen = set()
    while len(chosen) < 6:
        # 가중 추첨
        n = rng.choices(pool, weights=weights, k=1)[0]
        chosen.add(n)
    return tuple(sorted(chosen))

# 과거 페어 정규화 최대값 (스코어 스케일링)
max_pair_avg = max(pair_freq.values()) if pair_freq else 1

def total_score(combo):
    num_s = sum(score_num[n] for n in combo) / 6  # 0~1
    pair_s = pair_score(combo) / max_pair_avg     # 0~1 근사
    return num_s * 0.6 + pair_s * 0.4

# ---- 메인 ----
def main(target=10, attempts=200000, seed=42):
    rng = random.Random(seed)
    seen = set()
    cands = []
    tried = 0
    while tried < attempts and len(cands) < 5000:
        tried += 1
        combo = weighted_sample_combo(rng)
        if combo in seen:
            continue
        seen.add(combo)
        if not passes_filters(combo):
            continue
        if combo in past_sets:  # 과거 당첨 조합과 중복 → 제외
            continue
        cands.append(combo)

    cands.sort(key=lambda c: total_score(c), reverse=True)
    top = cands[:target]

    print(f"=== 후보 생성 시도: {tried}, 필터 통과: {len(cands)}, 추천: {len(top)} ===")
    print(f"[1221회 당첨번호] {sorted(LAST['numbers'])} + 보너스 {LAST['bonus']}\n")
    print(f"{'No.':<4}{'조합':<32}{'합계':<6}{'홀짝':<8}{'저고':<8}{'연속':<6}{'점수':<8}")
    for i, c in enumerate(top, 1):
        odd = sum(1 for n in c if n % 2 == 1)
        low = sum(1 for n in c if n <= 22)
        s_c = sorted(c)
        consec = sum(1 for j in range(5) if s_c[j + 1] - s_c[j] == 1)
        sc = total_score(c)
        combo_str = "-".join(f"{n:2d}" for n in c)
        print(
            f"{i:<4}{combo_str:<32}{sum(c):<6}{f'{odd}:{6-odd}':<8}"
            f"{f'{low}:{6-low}':<8}{consec:<6}{sc:.3f}"
        )

    # 과거 중복 검사 결과 요약
    print(f"\n[과거 회차 데이터 중복 검사] 1221회차 전체 정렬조합과 비교 → 추천 10개 모두 0 매치")

    # 추천 조합이 과거 어느 회차와 가장 비슷한지(공통 5개) 참고로 표시
    print("\n[참고: 추천 조합과 가장 유사한 과거 회차 (공통 5개 이상)]")
    found_any = False
    for i, c in enumerate(top, 1):
        cset = set(c)
        for d in DATA:
            inter = cset & set(d["numbers"])
            if len(inter) >= 5:
                print(f"  추천{i}: {sorted(c)} ↔ {d['round']}회차 {d['numbers']} (공통 {len(inter)})")
                found_any = True
                break
    if not found_any:
        print("  공통 5개 이상인 과거 회차 없음 (모두 충분히 차별화됨)")

if __name__ == "__main__":
    main()
