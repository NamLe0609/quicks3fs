#!/bin/sh

if [ -f /home/init_done.marker ]; then
    echo "MinIO buckets already initialized. Skipping."
    exit 0
fi

sleep 1;
/usr/bin/mc alias set dockerminio http://minio:9000 root beak-lamp-blind;
/usr/bin/mc mb dockerminio/test;
sizes="4KiB 8KiB 16KiB 32KiB 64KiB 128KiB 256KiB 512KiB 1MiB 2MiB 4MiB 8MiB 16MiB 32MiB 64MiB 128MiB 256MiB 512MiB 1GiB 2GiB 4GiB"

for size in $sizes; do
    filename="file-${size}.dat"
    head -c $size </dev/urandom > "/tmp/${filename}"    
    mc cp "/tmp/${filename}" dockerminio/test/
    rm "/tmp/${filename}"    
done

touch /home/init_done.marker
