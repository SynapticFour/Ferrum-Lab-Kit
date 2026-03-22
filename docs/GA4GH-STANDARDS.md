# GA4GH standards (Lab Kit mapping)

Lab Kit **selects and wires** Ferrum implementations; version pins follow **Ferrum** releases.

| Surface | Typical standard | What it enables for labs |
|---------|------------------|---------------------------|
| **Beacon v2** | GA4GH Beacon API v2 | ELIXIR Beacon Network, cohort discovery, GDI metadata exchange |
| **DRS** | GA4GH DRS | Stable object IDs over object storage |
| **htsget** | GA4GH htsget | Streaming genomic reads/variants without full file copies |
| **WES** | GA4GH WES | Workflow execution metadata & portability |
| **TES** | GA4GH TES | Task execution on HPC/K8s backends |
| **TRS** | GA4GH TRS | Tool/workflow registry (e.g. nf-core references) |
| **Passport / AAI** | GA4GH Passport | Controlled access, visas, institutional identity via LS Login |

Exact specification versions are pinned in Ferrum; update this table when Ferrum cuts releases.

## See also

- [Documentation index](README.md)  
- [GA4GH workflow primer](GA4GH-WORKFLOW-PRIMER.md) — control flow, DRS access patterns, engines, nested execution, platforms  
- [Operations checklist](OPERATIONS-CHECKLIST.md)
