#!/bin/zsh

d=${0:a:h:h}
json=`cat $d/gpt.json`
toml=`cat $d/Cargo.toml`
cd $d/src/
list=(`zsh -c "ls *.rs"`)

body="
今、AGE systemを作っているよ。どんなものかというと、jsonを参照してここにすべてが書かれている。

$json

リポジトリはこちらになる。
git.syui.ai:ai/gpt.git

内容はこんな感じ。

\`\`\`toml
$toml
\`\`\`

`
for i in $list; do
	if [ -f $d/src/$i ];then
		t=$(cat $d/src/$i)
		echo 
		echo '\`\`\`rust'
		echo $t
		echo '\`\`\`'
		echo 
	fi
done
`

次は何を実装すればいいと思う。
"

echo $body
