# Changelog

Todos los cambios notables de este proyecto serán documentados en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
y este proyecto se adhiere a [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-02-10

### Added
- **Monitor del Sistema**: Estadísticas en tiempo real del sistema
- **Batería**: Estado de carga, ciclos y salud de batería
- **Memoria**: Uso de memoria detallado y liberación
- **Procesos**: Lista y gestión de procesos en ejecución
- **Conexiones de Red**: Monitor de conexiones TCP/UDP activas
- **Firewall**: Estado y control del firewall de macOS
- **Bluetooth**: Gestión de dispositivos Bluetooth
- **Escáner de Puertos**: Detección de puertos abiertos y servicios activos
- **Limpieza del Sistema**: Limpieza de archivos temporales y cachés
- **Desinstalador**: Desinstalación completa de aplicaciones
- **Archivos Grandes**: Búsqueda de archivos que ocupan espacio
- **Archivos Duplicados**: Detección inteligente de duplicados
- **Archivos Huérfanos**: Limpieza de archivos sin referencias
- **Elementos de Inicio**: Gestión de apps y servicios al inicio
- **Homebrew**: Integración con el gestor de paquetes
- **Proyectos**: Gestión de proyectos de desarrollo

### Changed
- Optimización del rendimiento general
- Mejora en el uso de memoria
- Interfaz de usuario modernizada

### Fixed
- Bug en la actualización de estadísticas del sistema
- Problema con la búsqueda de procesos
- Error al desinstalar aplicaciones

### Security
- Validación mejorada de entradas de usuario
- Sanitización de rutas de archivo

### Documentation
- README.md completo con todas las funcionalidades
- Guía de contribución (CONTRIBUTING.md)
- Licencia MIT (LICENSE)

### Build
- Script de generación de iconos optimizado
- Script de creación de DMG automatizado
- Integración con GitHub Actions para CI/CD

## [0.1.0] - 2026-01-23

### Added
- Estructura inicial del proyecto
- Framework básico MVVM
- Integración con Swift Package Manager
- Icono generador inicial

---

[Unreleased]: https://github.com/DANO-AMP/MMAC/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/DANO-AMP/MMAC/releases/tag/v1.0.0
[0.1.0]: https://github.com/DANO-AMP/MMAC/releases/tag/v0.1.0