/*
I can't get rust to properly accessl linker variable. so this is the solution..
*/

extern int _stub_begin;
__attribute__ ((section(".stub"))) unsigned int* stub_begin_glue() {
        return (unsigned int*)&_stub_begin;
}

extern int _stub_end;
__attribute__ ((section(".stub"))) unsigned int* stub_end_glue() {
        return (unsigned int*)&_stub_end;
}

extern int _kernel_start_phy;
__attribute__ ((section(".stub"))) unsigned int* kernel_start_phy_glue() {
        return (unsigned int*)&_kernel_start_phy;
}

extern int _kernel_start_virt;
__attribute__ ((section(".stub"))) unsigned int* kernel_start_virt_glue() {
        return (unsigned int*)&_kernel_start_virt;
}

extern int _kernel_end_virt;
__attribute__ ((section(".stub"))) unsigned int* kernel_end_virt_glue() {
        return (unsigned int*)&_kernel_end_virt;
}

extern int l1pagetable;
__attribute__ ((section(".stub"))) unsigned int* l1pagetable_glue() {
        return (unsigned int*)&l1pagetable;
}


