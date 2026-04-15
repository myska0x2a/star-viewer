for dir in */
do
    dir=${dir%*/}
    echo "${dir##*/}"
    glslc $dir/*.vert -o $dir/*.vert.spv
done

