OUTPUT_ARCH(arm)
ENTRY(_start)
SECTIONS
{
	. = 0x8000;
	_physical_begin = . ;
	_begin =  . ;
    _stub_begin = .;

	.stub :
	AT (_stub_begin)
	{
		*(.stub)
	}
	.stub.data :
	{
		*(.stub.data)
	}
	.stub.data.pagegtable :
	{
		*(.stub.data.pagegtable)
	}
   /* _stub_end = ALIGN(0x1000);  hack till we improve the stub... kernel needs to be 1mb align*/
    _stub_end = ALIGN(0x100000);
	/* kernel virtual address! */
	. = 0xCC000000;
	_kernel_start_phy = _stub_end;
	_kernel_start_virt = .;

	.text :
	AT (_stub_end)
	{
		*(.text)
		*(.text.*)
	}
	_etext = . ;

	.rodata :
	AT (LOADADDR(.text) + (ADDR(.rodata) - ADDR(.text)))
	{
		*(.rodata)
		*(.rodata.*)
	}

    .data.rel.ro : 
	AT (LOADADDR(.text) + (ADDR(.data.rel.ro) - ADDR(.text))) {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }

	.initfini :
	AT (LOADADDR(.text) + (ADDR(.initfini) - ADDR(.text)))
	{
		__init_array_start = . ;
		*(SORT_BY_INIT_PRIORITY(.init_array.*))
		*(SORT_BY_INIT_PRIORITY(.ctors*))
		*(.init_array)
		__init_array_end = . ;
		__fini_array_start = . ;
		*(SORT_BY_INIT_PRIORITY(.fini_array.*))
		*(SORT_BY_INIT_PRIORITY(.dtors*))
		*(.fini_array)
		__fini_array_end = . ;
	}

	_data_start = .;
	.data :
	AT (LOADADDR(.text) + (ADDR(.data) - ADDR(.text)))
	{
		*(.data)
	}
	_edata = . ;

	.tdata :
	AT (LOADADDR(.text) + (ADDR(.tdata) - ADDR(.text)))
	{
		_tdata_start = . ;
		*(.tdata)
		_tdata_end = . ;
	}

	.tbss :
	AT (LOADADDR(.text) + (ADDR(.tbss) - ADDR(.text)))
	{
		_tbss_start = . ;
		*(.tbss)
		_tbss_end = . ;
	}

	__bss_start = . ;
	.bss :
	AT (LOADADDR(.text) + (ADDR(.bss) - ADDR(.text)))
	{

		__init_bss_start = . ;
		*(.bss)
		*(.bss.*)
		*(COMMON)
		*(.bootstack)
		__init_bss_end = . ;
	}
	__bss_end = . ;

	_end = . ;
	PROVIDE (end = .) ;
	_kernel_end_virt = end;

}