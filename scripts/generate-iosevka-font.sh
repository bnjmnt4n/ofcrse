#!/bin/sh

BASEDIR=$(dirname $0)

cd $BASEDIR
git clone --depth 1 https://github.com/be5invis/Iosevka.git
cp private-build-plans.toml Iosevka

pushd Iosevka
npm install
npm run build -- --jCmd=2 webfont::iosevka-custom
popd

cp Iosevka/dist/iosevka-custom/woff2/*.woff2 ../public/assets/fonts
