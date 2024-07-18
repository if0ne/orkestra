# How to use

1. Download Docker

2. **Change** password for Postgres in ``postgres-default.env``

3. **Change** password for Postgres in ``orkestra-auth-system/default.env``

4. **Change** host ip address in ``orkestra-server-manager/default.env`` for your public IP or stay ``0.0.0.0`` if you want to test local

5. **Change** ip address for Postgres in ``orkestra-auth-system/default.env``:
    * If you use **Windows** ip address will be ``host.docker.internal``
    * If you use **Linux** check ip address of compose bridge by ``docker network inspect orkestra_default``

6. ``docker compose up -d --build``
