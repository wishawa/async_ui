
export const style_m9f0qa6h = `
.wrapper-m9f0qa6h {
	overflow: scroll;
	max-height: 40em;
}
.table-m9f0qa6h {
	table-layout: fixed;
	border-collapse: collapse;
}
.table-m9f0qa6h tr, .table-m9f0qa6h td {
	padding: 0;
}
.cell-m9f0qa6h {
	position: relative;
}
.cell-m9f0qa6h > span {
	position: absolute;
	top: 0;
	bottom: 0;
	margin: auto;
	right: 2px;
	pointer-events: none;
}
.cell-m9f0qa6h:focus-within > span {
	visibility: hidden;
}
.cell-m9f0qa6h > input {
	border: 1px solid grey;
}
		`;
document.head.appendChild(document.createElement("style")).innerHTML = style_m9f0qa6h;
        