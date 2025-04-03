import init, { WasmApp } from './pkg/outline.js';

async function run() 
{
	await init();

	const canvas = document.getElementById( 'canvas' );
	canvas.width = window.innerWidth;
	canvas.height = window.innerHeight;

	try 
	{
		const app = new WasmApp( 'canvas', canvas.width, canvas.height );

		function renderLoop( timestamp ) 
		{
				app.render( timestamp );
				requestAnimationFrame( renderLoop );
		}

		requestAnimationFrame( renderLoop );

		window.addEventListener('resize', 
			() => 
			{
				canvas.width = window.innerWidth;
				canvas.height = window.innerHeight;
				app.resize( canvas.width, canvas.height );
			}
		);

	} 
	catch ( error ) 
	{
		console.error( "Failed to initialize WasmApp:", error );
		alert( `Failed to initialize WebGL App: ${error}\n\nPlease ensure your browser supports WebGL2 and the necessary extensions (like EXT_color_buffer_float).` );
	}
}

run();