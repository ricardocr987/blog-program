use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
/// Moving on to the #[program] macro below, this is where we define our program.
/// Each method inside here defines an RPC request handler (aka instruction handler) which can be invoked by clients
pub mod blog_program {
    use super::*;

    pub fn initialize_blog(
        // El primer parámetro para cada RPC handler/instrucción es el Context struct
        ctx: Context<InitializeBlog>, 
        blog_account_bump: u8 
    ) -> Result<()> {

        let blog = &mut ctx.accounts.blog_account; // declaramos una refecencia mutable del account

        blog.bump = blog_account_bump; // Almacenamos el bump!
        blog.authority = *ctx.accounts.authority.to_account_info().key; // * actúa para deshacer la referencia, es lo contrario a &
        blog.post_count = 0;

        Ok(())
    }

    pub fn create_post(
        ctx: Context<CreatePost>, 
        post_account_bump: u8, 
        title: String, 
        body: String
    ) -> Result<()> {

        let blog = &mut ctx.accounts.blog_account;
        let post = &mut ctx.accounts.post_account;

        post.authority = *ctx.accounts.authority.to_account_info().key;
        post.title = title;
        post.body = body;
        post.bump = post_account_bump;
        post.entry = blog.post_count;
       
        blog.post_count += 1;

        Ok(())
    }

    pub fn update_post(
        ctx: Context<UpdatePost>, 
        title: String, 
        body: String
    ) -> Result<()> {

        let post = &mut ctx.accounts.post_account;

        post.title = title;
        post.body = body;

        Ok(())
    }
}

#[derive(Accounts)]
// Puedes acceder a los argumentos de las instruccion con #[instruction(..)]
// Tiene que ser el mismo orden que en la instrucción, se suele usar para almacenar el bump 
// y para establecer el espacio que ocupa variables dinámicas (String y Vectores)
#[instruction(blog_account_bump: u8)]
pub struct InitializeBlog<'info> {
    #[account(
        init, // Permite inicializar la Account
        payer = authority, // Especifíca quien va a pagar el rent
        space = 8 /*discriminator*/ + 1 /*u8*/ + 1 /*u8*/ + 32 /*Pubkey*/,
        seeds = [ // Especifíca y comprueba las seeds para el PDA de la Account con init, 
            // combinado con init comprueba si el PDA ya existe
            b"blog".as_ref(),
            authority.key().as_ref(), // Esto es Rust: as_ref() es un método que nos permite pedir prestado una referencia
            // Léete esta capítulo del libro de Rust: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
        ],
        bump, // Valor que completa a las seeds para obtener un PDA válido
            // en tutoriales anteriores verás como antes se igualaba con la variable que hay en #[instruction(..)]
            // ahora ya se hace automáticamente
    )]
    blog_account: Account<'info, BlogAccount>,
    #[account(mut)]
    authority: Signer<'info>,
    system_program: Program<'info, System> // Es requerido cuando inicializamos una Account
        // Esto exige que nuestra Account deba ser propiedad de nuestro Program y que debe ser deserializado
        // como el struct BlogAccount de abajo en #[account] 
}

#[derive(Accounts)]
#[instruction(post_account_bump: u8, title: String, body: String, vector_capacity: u16)]
pub struct CreatePost<'info> {
    #[account(
        init, 
        payer = authority, 
        seeds = [
            b"post".as_ref(), // as_ref() es un método que convierte código genérico en una referencia
            blog_account.key().as_ref(),
            &[blog_account.post_count as u8].as_ref(),
        ],
        bump, 
        space = PostAccount::space(&title, &body, vector_capacity), // Dice cuando espacio va a ocupar
                    // la Account, necesario para calcular la cantidad de rent-exemption
                    // llamamos a la función space
    )]
    post_account: Account<'info, PostAccount>,
    // Todas las validaciones son manejadas por #[account(..)] macros, permitiéndonos centrarnos en la lógica o en el diseño de las instrucciones
    #[account(mut, has_one = authority)] //
    blog_account: Account<'info, BlogAccount>,
    #[account(mut)]
    authority: Signer<'info>,
    system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(title: String, body: String)]
pub struct UpdatePost<'info> {
    #[account(mut, has_one = authority)]
    blog_account: Account<'info, BlogAccount>,
    #[account(mut, has_one = authority)]
    post_account: Account<'info, PostAccount>,
    #[account(mut)]
    authority: Signer<'info>,
}

/// Aquí definimos nuetro Blog Account
/// Definimos un struct que tiene tres propiedades públicas: bump, post_count y authority
/// Esta Account será pasada como argumento a las instrucciones si la incluimos en el Context
#[account]
#[derive(Default)] // Ya que no tenemos ninguna variable dinámica (String, Vector), incluimos este macro
pub struct BlogAccount {
    pub bump: u8, // int de 1byte/8bits, bump: Nos dice el valor que completa a las seeds para obtener un PDA válido
    pub post_count: u8,  // int para almacenar el número de posts que tiene cada blog
    pub authority: Pubkey, // almacenamos la Pubkey del usuario propietario del blog
}


#[account]
pub struct PostAccount{
    pub authority: Pubkey,
    pub bump: u8,
    pub entry: u8,
    pub title: String,
    pub body: String,
    pub vector_capacity: u16, // Guarmos la capacidad máxima del vector
    pub likes_pubkeys: Vec<Pubkey>, // Guardamos las pubKeys de los usuarios que les guste el post
}

impl PostAccount{ // Cada struct puede tener múltiple impl blocks, básicamente puedes declarar funciones
    pub fn space(title: &str, body: &str, capacity: u16) -> usize{ // Esta función devuelve un integer unsigned (sólo permite positivos)
        // los strings son referencias, léete esta capítulo del libro de Rust: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
        8 + // Discrimanator: son 8 bytes que Anchor pone al frente de una cuenta, como una cabecera. Permite a Anchor saber 
            // qué tipo de cuenta debe deserializar los datos.
        32 + // Pubkey
        1 + // u8
        1 + // u8
        4 + title.len() + // el 4 es otro discriminator + la longitud de la cadena, esto genera un problema, una vez que la Account es creada
                        // el usuario no podría poner un título más largo de lo que puso en un primer momento
        4 + body.len() +
        4 + (capacity as usize) * std::mem::size_of::<Pubkey>()
        // Para sacar el espacio que ocupa un vector tenemos que multiplacar la capacidad que asignamos nosotros por el espacio de la variable que 
        // queremos vectorizar. La capacidad representa cuantas posiciones podemos almacenar dentro del vector.
    }
}
