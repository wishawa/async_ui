
export const style_odi9mfip = `
.top-part-odi9mfip {
	display: flex;
	flex-direction: row;
}
.area-odi9mfip {
	position: relative;
	height: 400px;
	width: 600px;
	border: 1px solid black;
	overflow: hidden;
}
.circle-odi9mfip {
	position: absolute;
	pointer-events: none;
	height: 48px;
	width: 48px;
	border-radius: 25px;
	border: 1px solid black;
	--circle-scale: 1.0;
	transform: translate(-50%, -50%) scale(var(--circle-scale));
	z-index: 50;
}
.circle-selected-odi9mfip {
	background-color: rgba(0,0,0,0.2);
}
.backdrop-odi9mfip {
	position: absolute;
	left: 0;
	right: 0;
	top: 0;
	bottom: 0;
}
.popup-odi9mfip {
	position: absolute;
	z-index: 100;
	background-color: white;
	box-shadow: 0 0 5px 0 black;
	border-radius: 4px;
}
.popup-odi9mfip > input {
	margin: 4px;
}
		`;
document.head.appendChild(document.createElement("style")).innerHTML = style_odi9mfip;
        