#!/bin/sh

sleep 1;
/usr/bin/mc alias set dockerminio http://minio:9000 root beak-lamp-blind;
/usr/bin/mc mb dockerminio/test;
sizes="4KB 8KB 16KB 32KB 64KB 128KB 256KB 512KB 1MB 2MB 4MB 8MB 16MB 32MB 64MB 128MB 256MB 512MB 1GB 2GB 4GB 8GB"

for size in $sizes; do
    filename="file-${size}.dat"
    echo "Generating ${filename}..."
    /usr/bin/mc data generate "/tmp/${filename}" "${size}"
    /usr/bin/mc cp "/tmp/${filename}" dockerminio/test/
    rm "/tmp/${filename}"
done
