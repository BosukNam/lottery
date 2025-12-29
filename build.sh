#!/bin/bash

set -e

echo "🔨 WASM 빌드 시작..."

# wasm-pack 설치 확인
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack이 설치되어 있지 않습니다."
    echo "설치 명령: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# WASM 빌드
echo "📦 Rust → WASM 컴파일 중..."
wasm-pack build --target web --out-dir www/pkg

# JSON 데이터 복사
echo "📄 데이터 파일 복사 중..."
cp lottery_data.json www/

# docs 디렉토리 생성 (GitHub Pages용)
echo "📁 docs 디렉토리 생성 중..."
rm -rf docs
mkdir -p docs
cp -r www/* docs/

echo "✅ 빌드 완료!"
echo "📍 로컬 테스트: python3 -m http.server --directory www 8000"
echo "🌐 브라우저에서 http://localhost:8000 접속"
