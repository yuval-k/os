OUTPUT_ARCH(arm)
ENTRY(_start)
SECTIONS
{
	. = 0;
	_physical_begin = . ;
/*	TODO move stub stuff to stub section .stub:
    and have the kernel be in a virtual address
 */

	_begin =  . ;
	/* Set virtual address here.
	like so:
	 . = 0xC0008000;
	 */
	.text :
	AT (_begin)
	{
		*(.start)
		*(.text)
		*(.text.*)
		*(.stub)
		*(.note*)
	}
	_etext = . ;

	.rodata :
	AT (LOADADDR(.text) + (ADDR(.rodata) - ADDR(.text)))
	{
		*(.rodata)
		*(.rodata.*)
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
	_end = . ;
	PROVIDE (end = .) ;
}