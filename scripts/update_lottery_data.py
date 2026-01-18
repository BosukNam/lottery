#!/usr/bin/env python3
"""
동행복권 API를 통해 로또 당첨번호를 자동으로 업데이트하는 스크립트

Playwright를 사용하여 실제 브라우저로 봇 차단 우회
"""

import json
import time
from pathlib import Path

API_URL = "https://www.dhlottery.co.kr/common.do?method=getLottoNumber&drwNo={}"

# 프로젝트 루트 기준 lottery_data.json 파일 경로들
DATA_FILES = [
    "docs/lottery_data.json",
    "lottery_data.json",
    "www/lottery_data.json",
]


def get_project_root():
    """프로젝트 루트 디렉토리 반환"""
    script_dir = Path(__file__).parent
    return script_dir.parent


def load_lottery_data(filepath):
    """lottery_data.json 파일 로드"""
    with open(filepath, "r", encoding="utf-8") as f:
        return json.load(f)


def save_lottery_data(filepath, data):
    """lottery_data.json 파일 저장"""
    with open(filepath, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)


def fetch_with_playwright(round_no):
    """Playwright를 사용하여 API 호출 (봇 차단 우회)"""
    try:
        from playwright.sync_api import sync_playwright

        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context(
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"
            )
            page = context.new_page()

            url = API_URL.format(round_no)
            print(f"[Playwright] {url} 접속 중...")

            response = page.goto(url, wait_until="networkidle", timeout=30000)
            content = page.content()

            browser.close()

            # JSON 응답 추출
            if content.startswith("<!DOCTYPE") or content.startswith("<html"):
                # HTML 페이지인 경우 body 내용에서 JSON 추출 시도
                try:
                    # pre 태그 안의 JSON 찾기
                    import re
                    json_match = re.search(r'\{[^{}]*"returnValue"[^{}]*\}', content)
                    if json_match:
                        return json.loads(json_match.group())
                except:
                    pass
                print(f"[Playwright] HTML 응답 - 첫 200자: {content[:200]}")
                return None

            return json.loads(content)

    except ImportError:
        print("[ERROR] Playwright가 설치되지 않았습니다.")
        return None
    except Exception as e:
        print(f"[Playwright] 오류: {e}")
        return None


def fetch_with_requests(round_no):
    """requests를 사용한 기본 API 호출"""
    try:
        import requests

        headers = {
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            "Accept": "application/json, text/javascript, */*; q=0.01",
            "Referer": "https://www.dhlottery.co.kr/gameResult.do?method=byWin",
        }

        url = API_URL.format(round_no)
        response = requests.get(url, headers=headers, timeout=15)

        print(f"[requests] 상태 코드: {response.status_code}")
        print(f"[requests] Content-Type: {response.headers.get('Content-Type', '')}")

        if "application/json" in response.headers.get("Content-Type", ""):
            return response.json()

        # JSON으로 파싱 시도
        try:
            return response.json()
        except:
            print(f"[requests] HTML 응답 감지")
            return None

    except Exception as e:
        print(f"[requests] 오류: {e}")
        return None


def fetch_lottery_result(round_no, max_retries=3):
    """동행복권 API에서 특정 회차 당첨번호 조회"""

    for attempt in range(max_retries):
        # 먼저 Playwright 시도
        print(f"\n[시도 {attempt + 1}/{max_retries}] 회차 {round_no} 조회...")

        data = fetch_with_playwright(round_no)

        if data is None:
            # Playwright 실패 시 requests로 폴백
            data = fetch_with_requests(round_no)

        if data and data.get("returnValue") == "success":
            return {
                "round": data["drwNo"],
                "numbers": sorted([
                    data["drwtNo1"],
                    data["drwtNo2"],
                    data["drwtNo3"],
                    data["drwtNo4"],
                    data["drwtNo5"],
                    data["drwtNo6"],
                ]),
                "bonus": data["bnusNo"],
            }
        elif data and data.get("returnValue") == "fail":
            print(f"[DEBUG] 회차 {round_no} - returnValue: fail (추첨 전)")
            return None

        if attempt < max_retries - 1:
            wait_time = 2 ** (attempt + 1)
            print(f"{wait_time}초 후 재시도...")
            time.sleep(wait_time)

    return None


def get_latest_round(data):
    """현재 데이터에서 최신 회차 번호 반환"""
    if not data:
        return 0
    return max(item["round"] for item in data)


def update_lottery_data():
    """로또 데이터 업데이트 메인 함수"""
    project_root = get_project_root()

    # 메인 데이터 파일 경로 (docs/lottery_data.json)
    main_data_file = project_root / DATA_FILES[0]

    if not main_data_file.exists():
        print(f"데이터 파일을 찾을 수 없습니다: {main_data_file}")
        return False

    # 현재 데이터 로드
    data = load_lottery_data(main_data_file)
    latest_round = get_latest_round(data)
    print(f"현재 최신 회차: {latest_round}")

    # API 연결 검증 (기존 회차 테스트)
    print(f"\n[INFO] API 연결 검증 중 (회차 {latest_round} 테스트)...")
    test_result = fetch_lottery_result(latest_round, max_retries=2)

    if test_result:
        print(f"[INFO] API 연결 검증 성공: {test_result['numbers']}")
    else:
        print("[WARN] API 검증 실패. 새 회차 조회를 시도합니다...")

    # 새로운 회차 확인 및 추가
    new_rounds_added = 0
    next_round = latest_round + 1

    time.sleep(1)

    while True:
        print(f"\n회차 {next_round} 조회 중...")
        result = fetch_lottery_result(next_round)

        if result is None:
            print(f"회차 {next_round}은 아직 추첨되지 않았거나 조회할 수 없습니다.")
            break

        data.append(result)
        new_rounds_added += 1
        print(f"회차 {next_round} 추가됨: {result['numbers']} + 보너스 {result['bonus']}")
        next_round += 1

        time.sleep(0.5)

    if new_rounds_added == 0:
        print("\n새로운 회차가 없습니다. 데이터가 최신 상태입니다.")
        return False

    # 회차 순으로 정렬
    data.sort(key=lambda x: x["round"])

    # 모든 데이터 파일에 저장
    for data_file in DATA_FILES:
        filepath = project_root / data_file
        if filepath.exists() or data_file == DATA_FILES[0]:
            save_lottery_data(filepath, data)
            print(f"저장됨: {filepath}")

    print(f"\n총 {new_rounds_added}개의 새 회차가 추가되었습니다.")
    print(f"현재 최신 회차: {get_latest_round(data)}")
    return True


if __name__ == "__main__":
    update_lottery_data()
    exit(0)
