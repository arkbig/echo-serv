#!/usr/bin/env sh
#====================================================================
set -eu
# set -eux
umask 0022
PATH='.:/usr/sbin:/usr/bin:/sbin:/bin'
IFS=$(printf ' \t\n_')
IFS=${IFS%_}
export IFS LC_ALL=C LANG=C PATH
#--------------------------------------------------------------------

# Adjust UID,GID
uid=$(stat -c "%u" .)
gid=$(stat -c "%g" .)
ug_name=echo_serv
if [ "${CONTAINER_GID}" != "${gid}" ]; then
    groupmod -g "${CONTAINER_GID}" -o "${ug_name}"
    chgrp -R "${CONTAINER_GID}" .
fi
if [ "${CONTAINER_UID}" != "${uid}" ]; then
    usermod -g "${ug_name}" -o -u "${CONTAINER_UID}" "${ug_name}"
    chown -R "${CONTAINER_UID}" .
fi

# Run as
if [ "$(id -u)" = "${CONTAINER_UID}" ]; then
    exec "$@"
else
    exec su-exec "${ug_name}" "$@"
fi
