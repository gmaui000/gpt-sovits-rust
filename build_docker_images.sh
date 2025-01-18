#!/bin/bash
set -e

force="false"
option=""
while getopts t:f opt
do
	case $opt in
		t)
			type=$OPTARG
			;;
		f)
			force="true"
			;;
		?)
			echo "unkonwn"
			exit
			;;
       esac
done

help_string=".sh [-t build|push] [-f]"

if [[ ! -n $type ]];then
    echo $help_string
    exit
fi

if [[ $force = "true" ]]; then
        option="--no-cache"
fi

platform=`arch`
echo "auto select arch:${platform}" 

#镜像仓库地址
repository="crpi-6ty60rr45g7d9i1q.cn-shanghai.personal.cr.aliyuncs.com"
#仓库名称
namespace="gmaui000"
#项目名称
packagename="tts_server"

imagename=$repository/$namespace/$packagename

datetime=$(date +%Y%m%d)
case $type in
	'build')
		if [ -e "target/release/tts_server" ]; then
			cargo build --release --bin tts_server
			docker buildx build $option --platform=$platform --network=host -f docker/dockerfile.nobuild -t $imagename:$datetime .
		else
		    docker buildx build $option --platform=$platform --network=host -f docker/dockerfile -t $imagename:$datetime .
		fi
        ;;
	'push')
		echo "push to dst registry"
		image=$(docker image ls --filter "reference=$imagename" --quiet | uniq)
		echo "push $imagename:$datetime"
		docker push $imagename:$datetime
		docker tag $imagename:$datetime $imagename:latest
		docker push $imagename:latest
		docker rmi -f $image
		;;
 	*)
		echo "unkonwn type"
		echo $help_string
		exit
		;;
esac
